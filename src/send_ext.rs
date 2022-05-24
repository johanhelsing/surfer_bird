use bevy::prelude::*;
use surf::RequestBuilder;

use crate::RequestBundle;

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
    commands.spawn_bundle(RequestBundle::<T>::new(request));
}
