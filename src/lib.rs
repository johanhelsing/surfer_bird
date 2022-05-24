#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use bevy::{ecs::query::WorldQuery, prelude::*};
use details::{Request, Response};
use serde::Deserialize;
use surf::StatusCode;
use systems::*;

mod send_ext;
mod systems;

pub mod prelude {
    pub use super::{send_ext::SurferSendExt, CompletedRequest, RequestBundle, SurferPlugin};
}

pub(crate) mod details {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Request {
        pub(crate) inner: surf::Request,
    }

    #[derive(Component)]
    pub struct Response(pub(crate) Result<(surf::Response, String), surf::Error>);
}

mod resources {
    use bevy::prelude::*;

    #[derive(Default, Deref, DerefMut)]
    pub(crate) struct HttpClient(surf::Client);
}

pub struct SurferPlugin;

impl Plugin for SurferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::HttpClient>();
        app.add_event::<details::Request>();
        app.add_system(send_requests);
        app.add_system(extract_responses);
    }
}

#[derive(Component)]
struct ResponseMarker<T: 'static> {
    marker: PhantomData<T>,
}

unsafe impl<T> Sync for ResponseMarker<T> {}
unsafe impl<T> Send for ResponseMarker<T> {}

impl<T> Default for ResponseMarker<T> {
    fn default() -> Self {
        Self { marker: default() }
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct CompletedRequest<'w, T: 'static> {
    marker: &'w mut ResponseMarker<T>,
    request: &'w mut Request,
    raw_response: &'w mut Response,
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Http error")]
    Http {
        status: surf::StatusCode,
        type_name: Option<String>, // perf: make cow?
    },
    #[error("Http status error")]
    Status(surf::StatusCode),
    #[error("Json error")]
    Json(#[from] serde_json::Error),
}

impl From<surf::Error> for RequestError {
    fn from(err: surf::Error) -> Self {
        Self::Http {
            type_name: err.type_name().map(|t| t.to_owned()),
            status: err.status(),
        }
    }
}

impl<'a> From<&surf::Error> for RequestError {
    fn from(err: &surf::Error) -> Self {
        Self::Http {
            status: err.status(),
            type_name: err.type_name().map(|t| t.to_owned()),
        }
    }
}

impl From<StatusCode> for RequestError {
    fn from(code: StatusCode) -> Self {
        Self::Status(code)
    }
}

impl<'w, T: Deserialize<'w>> CompletedRequestReadOnlyItem<'w, T> {
    pub fn data(&self) -> Result<T, RequestError> {
        // perf: deserializing here is maybe not ideal? May cause hickups for large payloads?
        let (response, body) = self.raw_response.0.as_ref()?;

        let status = response.status();

        if !status.is_success() {
            Err(response.status())?
        }
        debug!("body {body}, response code {}", response.status());
        Ok(serde_json::from_str(body)?)
    }

    pub fn body_string(&self) -> Result<&str, RequestError> {
        let (_response, body) = self.raw_response.0.as_ref()?;
        Ok(body)
    }
}

#[derive(Bundle)]
pub struct RequestBundle<T: 'static> {
    request: Request,
    marker: ResponseMarker<T>,
}

impl<T: 'static> RequestBundle<T> {
    pub fn new(request: surf::Request) -> Self {
        let request = Request { inner: request };
        Self {
            request,
            marker: default(),
        }
    }
}

impl<T> From<surf::RequestBuilder> for RequestBundle<T> {
    fn from(request_builder: surf::RequestBuilder) -> Self {
        Self::from(request_builder.build())
    }
}

impl<T> From<surf::Request> for RequestBundle<T> {
    fn from(request: surf::Request) -> Self {
        Self::new(request)
    }
}
