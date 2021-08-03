use futures::Future;
use lapin::Channel;
use log::debug;
use log::info;
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::fmt::Display;

use crate::rmq_primitive;
use crate::rmq_primitive::Responsibility;
use crate::Error;
use crate::Result;

#[derive(Clone)]
pub struct HandlerState<InputMsg, UserError, AcceptFailureResult> {
    pub accept_failure: fn(&InputMsg, UserError) -> AcceptFailureResult,
    pub output_queues: Vec<String>,
}

pub async fn fan_out<InputMsg, UserError, AcceptFailureResult>(
    state: HandlerState<InputMsg, UserError, AcceptFailureResult>,
    channel: Channel,
    msg: InputMsg,
) -> Result<Responsibility>
where
    InputMsg: Serialize + DeserializeOwned + Display + Send + 'static,
    UserError: Display + From<serde_json::Error>,
    AcceptFailureResult: Future<Output = std::result::Result<(), UserError>>,
{
    let accept_failure = state.accept_failure;
    let output_queues = state.output_queues;
    match serde_json::to_vec(&msg) {
        Err(serde_error) => {
            // serialization errors are final, accept the failure
            warn!("failed to serialize message {}, accepting failure", &msg);
            // failure to accept will be retried
            let () = accept_failure(&msg, serde_error.into())
                .await
                .map_err(|ue| Error::UserError {
                    error: ue.to_string(),
                })?;
            Ok(Responsibility::Accept)
        }
        Ok(payload) => {
            // since there is no multi-queue transaction support in RabbitMQ,
            // we just send to the output queues one by one,
            // if any one of these failed, all of them will be redelivered,
            // however since we are dealing with at least once delivery,
            // this is acceptable, albeit annoying
            for output_queue in output_queues {
                // TODO @incomplete: do not publish delayed
                rmq_primitive::publish_delayed(
                    Some(channel.clone()),
                    &output_queue,
                    payload.clone(),
                )
                .await?;
            }
            Ok(Responsibility::Accept)
        }
    }
}
