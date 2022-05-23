use bevy::prelude::*;
use serde::Deserialize;
use surfer_bird::prelude::*;

fn main() {
    App::new()
        .add_plugin(SurferPlugin)
        .add_plugins(DefaultPlugins)
        .add_startup_system(make_request)
        .add_system(response_handler)
        .run();
}

fn make_request(mut commands: Commands) {
    surf::get("https://httpbin.org/json").surfer_send::<Response>(&mut commands);
}

fn response_handler(requests: Query<CompletedRequest<Response>>) {
    for request in requests.iter() {
        match request.data() {
            Ok(data) => {
                info!("Got slideshow {:#?}", data.slideshow);
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    slideshow: SlideShow,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SlideShow {
    author: String,
    date: String,
    slides: Vec<Slide>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Slide {
    title: String,
    r#type: String,
    items: Option<Vec<String>>,
}
