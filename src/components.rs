use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Deref, DerefMut)]
pub(crate) struct Request(pub surf::Request);

#[derive(Component, Deref, DerefMut)]
pub(crate) struct Response(pub(crate) Result<(surf::Response, String), surf::Error>);

#[derive(Component)]
pub(crate) struct ResponseMarker<T: 'static> {
    marker: PhantomData<T>,
}

unsafe impl<T> Sync for ResponseMarker<T> {}
unsafe impl<T> Send for ResponseMarker<T> {}

impl<T> Default for ResponseMarker<T> {
    fn default() -> Self {
        Self { marker: default() }
    }
}

#[derive(Component)]
pub struct LogErrors;
