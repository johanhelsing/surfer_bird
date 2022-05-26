use crate::{
    components::*,
    resources::{JobQueue, JobQueueReceiver},
};
use bevy::{prelude::*, tasks::IoTaskPool};
use futures_channel::oneshot;
use futures_util::StreamExt;

pub(crate) fn startup(
    mut job_queue_receiver: ResMut<JobQueueReceiver>,
    task_pool: Res<IoTaskPool>,
) {
    let job_queue_receiver = job_queue_receiver.take().unwrap();

    task_pool
        .spawn(async move {
            let client = surf::Client::default();
            let client = &client;

            const MAX_CONCURRENT_REQUESTS: usize = 100;

            job_queue_receiver
                .for_each_concurrent(MAX_CONCURRENT_REQUESTS, |job| async move {
                    let (request, sender) = job;
                    let result = async move {
                        let mut response = client.send(request).await?;
                        let body = response.body_string().await?;
                        Ok((response, body))
                    }
                    .await;
                    let _ = sender.send(result);
                    // todo: select magic
                })
                .await;
        })
        .detach();
}

pub(crate) fn send_requests(
    completed_requests: Query<Entity, (With<Request>, With<Response>)>,
    mut commands: Commands,
    new_requests: Query<(Entity, &Request), (Without<RequestTask>, Without<Response>)>,
    job_queue: Res<JobQueue>,
) {
    for entity in completed_requests.iter() {
        commands.entity(entity).despawn();
    }

    for (entity, request) in new_requests.iter() {
        let request = request.0.clone();
        debug!("{} {}", request.method(), request.url());
        let (sender, receiver) = oneshot::channel();

        job_queue
            .0
            .unbounded_send((request, sender))
            .expect("failed to send http request job");

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
