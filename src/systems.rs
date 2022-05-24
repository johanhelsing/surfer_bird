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
        let request = request.inner.clone();
        debug!("{} {}", request.method(), request.url());
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
            commands
                .entity(entity)
                .insert(Response(response))
                .remove::<RequestTask>();
        }
    }
}
