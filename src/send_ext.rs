use bevy::prelude::*;
use surf::RequestBuilder;

use crate::{details::Request, ResponseMarker};

pub trait SurferSendExt {
    fn surfer_send<T: 'static>(self, commands: &mut Commands);
}

impl SurferSendExt for RequestBuilder {
    fn surfer_send<T: 'static>(self, commands: &mut Commands) {
        self.build().surfer_send::<T>(commands);
    }
}

impl SurferSendExt for surf::Request {
    fn surfer_send<T: 'static>(self, commands: &mut Commands) {
        surf_request::<T>(commands, self);
    }
}

fn surf_request<T: 'static>(commands: &mut Commands, request: surf::Request) {
    let request = Request { inner: request };
    // could just be a bundle instead?
    commands.spawn_bundle((request, ResponseMarker::<T>::default()));
}
