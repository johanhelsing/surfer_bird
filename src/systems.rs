use crate::{components::*, resources::HttpClient};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;

type RequestTask = Task<Result<(surf::Response, String), surf::Error>>;

pub(crate) fn send_requests(
    completed_requests: Query<Entity, (With<Request>, With<Response>)>,
    mut commands: Commands,
    new_requests: Query<(Entity, &Request), (Without<RequestTask>, Without<Response>)>,
    task_pool: Res<IoTaskPool>,
    client: Res<HttpClient>,
) {
    for entity in completed_requests.iter() {
        commands.entity(entity).despawn();
    }

    //perf: this spawns a lot of tasks... maybe better to have one worker task?

    for (entity, request) in new_requests.iter() {
        let client = (*client).clone();
        let request = request.0.clone();
        debug!("{} {}", request.method(), request.url());
        let task: RequestTask = task_pool.spawn(async move {
            let mut response = client.send(request).await?;
            let body = response.body_string().await?;
            Ok((response, body))
        });

        commands.entity(entity).insert(task);
    }
}

pub(crate) fn extract_responses(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RequestTask)>,
) {
    for (entity, mut task) in query.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut *task)) {
            commands
                .entity(entity)
                .insert(Response(response))
                .remove::<RequestTask>();
        }
    }
}

pub(crate) fn log_errors(
    completed_requests: Query<(&Request, &Response), (Added<Response>, With<LogErrors>)>,
) {
    for (request, response) in completed_requests.iter() {
        match response.0.as_ref() {
            Ok((response, body)) => {
                let status = response.status();
                if !status.is_success() {
                    let url = request.0.url();
                    let canonical_reason = status.canonical_reason();
                    error!("{status} {canonical_reason}: {url}\n {body}");
                }
            }
            Err(err) => {
                let url = request.0.url();
                error!("request failed: {err:?}, url: {url}");
            }
        }
    }
}
