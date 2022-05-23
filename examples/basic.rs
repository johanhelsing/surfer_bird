use bevy::prelude::*;
use serde::Deserialize;
use surf::RequestBuilder;
use surfer_bird::prelude::*;

struct ApiClient {
    token: String,
    base_url: String,
}

#[derive(Deserialize)]
struct Wrapper<T> {
    r#match: T,
}

#[derive(Deserialize, Debug)]
struct MatchResponse {
    id: String,
}

impl ApiClient {
    pub fn post(&self, url: &str) -> RequestBuilder {
        let req = surf::post(format!("{}{}", self.base_url, url));
        self.with_auth(req)
    }

    fn with_auth(&self, request: RequestBuilder) -> RequestBuilder {
        request.header("Authorization", format!("Bearer {}", self.token))
    }
}

fn main() {
    App::new()
        .add_plugin(SurferPlugin)
        .add_plugins(DefaultPlugins)
        .insert_resource(ApiClient {
            token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI5ODhlZmI5OS05ZTIwLTRlNjItOWVhMi04ZDI5MTFmZDgyMjIiLCJzZXNzaW9uIjoiYzExYjQ2MWItN2JiYy00MDdmLTk1YTYtMzMzNjE0ODNkMzVkIiwiaWF0IjoxNjUzMzE1MDk3LCJleHAiOjE2NTM0MDE0OTcsImh0dHBzOi8vaGFzdXJhLmlvL2p3dC9jbGFpbXMiOnsieC1oYXN1cmEtZGVmYXVsdC1yb2xlIjoidXNlciIsIngtaGFzdXJhLWFsbG93ZWQtcm9sZXMiOlsidXNlciJdLCJ4LWhhc3VyYS11c2VyLWlkIjoiOTg4ZWZiOTktOWUyMC00ZTYyLTllYTItOGQyOTExZmQ4MjIyIiwieC1oYXN1cmEtZGV2aWNlLWlkIjoiM2ZmMTM5MmItYTg2MS00YjY1LThjYTgtMjBjYWM2MTZjZWM4IiwieC1oYXN1cmEtZ2FtZSI6InRhaWxzIn19.XKLztRSJorQpm31V9XHBCsiC3Z_8qSRfS2tU2SGebK4".to_string(),
            base_url: "http://localhost:8080/api/rest/".to_string(),
        })
        .add_startup_system(make_request)
        .add_system(response_handler)
        .run();
}

fn make_request(mut commands: Commands, api: Res<ApiClient>) {
    info!("Making request");
    api.post("tails/match/get_or_create/public")
        .surfer_send::<Wrapper<MatchResponse>>(&mut commands);
}

fn response_handler(requests: Query<CompletedRequest<Wrapper<MatchResponse>>>) {
    for request in requests.iter() {
        match request.data() {
            Ok(data) => {
                info!("response_handler {:?}", data.r#match.id);
            }
            Err(err) => {
                error!("{}", err);
            }
        }
    }
}
