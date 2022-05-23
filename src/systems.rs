use crate::{
    details::{Request, Response},
    resources::HttpClient,
};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;

type RequestTask = Task<Result<(surf::Response, String), surf::Error>>;

// pub fn send_requests(
//     mut commands: Commands,
//     mut request_events: EventReader<Request>,
//     task_pool: Res<IoTaskPool>,
// ) {
//     for request in request_events.iter() {
//         let url = request.url.clone(); // maybe use queue instead of events to avoid cloning here
//         info!("todo make request to {url}");
//         let task = task_pool.spawn(async move {
//             surf::post(format!("http://localhost:8080/api/rest/{url}"))
//                 // .header("Authorization", format!("Bearer {}", session.token))
//                 .send()
//                 .await
//         });
//         commands.spawn().insert(task).insert(ResponseMarker<T>);
//     }
// }

pub(crate) fn send_requests(
    mut commands: Commands,
    new_requests: Query<(Entity, &Request), (Without<RequestTask>, Without<Response>)>,
    task_pool: Res<IoTaskPool>,
    client: Res<HttpClient>,
) {
    //perf: this spawns a lot of tasks... maybe better to have one worker task?

    for (entity, request) in new_requests.iter() {
        let client = (*client).clone();
        let request = request.inner.clone();
        info!("Sending request {}", request.url());
        let task: RequestTask = task_pool.spawn(async move {
            let mut response = client.send(request).await?;
            let body = response.body_string().await?;
            Ok((response, body))
        });

        commands.entity(entity).insert(task);
    }
}

pub fn extract_responses(mut commands: Commands, mut query: Query<(Entity, &mut RequestTask)>) {
    for (entity, mut task) in query.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut *task)) {
            info!("Received response");
            commands
                .entity(entity)
                .insert(Response(response))
                .remove::<RequestTask>();
        }
    }
}
