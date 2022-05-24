use crate::{components::*, resources::HttpClient};
use bevy::{prelude::*, tasks::IoTaskPool};
use futures_channel::oneshot;

#[derive(Component)]
pub(crate) struct RequestTask(oneshot::Receiver<Result<(surf::Response, String), surf::Error>>);

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
        let (sender, receiver) = oneshot::channel();

        task_pool
            .spawn(async move {
                let result = async move {
                    let mut response = client.send(request).await?;
                    let body = response.body_string().await?;
                    Ok((response, body))
                }
                .await;
                let _ = sender.send(result);
            })
            .detach();

        commands.entity(entity).insert(RequestTask(receiver));
    }
}

pub(crate) fn extract_responses(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RequestTask)>,
) {
    for (entity, mut task) in query.iter_mut() {
        if let Some(response) = task.0.try_recv().expect("sender dropped unexpectedly") {
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
