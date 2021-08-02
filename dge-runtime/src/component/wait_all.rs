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

/// Status of merging multiple messages into one `MergedMsg`
pub enum MergeStatus<MergedMsg> {
    /// The merge is incomplete, need more incoming messages.
    Partial,
    /// For the first time, the multiple input messages are merged into one.
    FreshMerge(MergedMsg),
    /// The incoming messages are already merged previously.
    AlreadyMerged,
}

pub struct HandlerState<InputMsg, MergeResult, UserError, AcceptFailureResult> {
    merge_messages: fn(&InputMsg) -> MergeResult,
    accept_failure: fn(&InputMsg, UserError) -> AcceptFailureResult,
    output_queue: Option<String>,
}

pub async fn wait_all<InputMsg, MergedMsg, UserError, MergeResult, AcceptFailureResult>(
    state: HandlerState<InputMsg, MergeResult, UserError, AcceptFailureResult>,
    channel: Channel,
    msg: InputMsg,
) -> Result<Responsibility>
where
    InputMsg: DeserializeOwned + Display + Send + 'static,
    MergedMsg: Serialize + Send + 'static,
    UserError: Display + From<serde_json::Error>,
    MergeResult: Future<Output = std::result::Result<MergeStatus<MergedMsg>, UserError>>,
    AcceptFailureResult: Future<Output = std::result::Result<(), UserError>>,
{
    let merge_messages = state.merge_messages;
    let accept_failure = state.accept_failure;
    let output_queue = state.output_queue;
    match merge_messages(&msg).await {
        Err(user_error) => {
            // user handler returned error,
            // since this may be a transient error (i.e. db op), we retry it.
            // this behaviour is strictly for convenience
            warn!(
                "failed to merge messages for {}, error is: {}, will be retried",
                &msg, user_error
            );
            // reject the message to retry
            Ok(Responsibility::Reject)
        }
        Ok(MergeStatus::Partial) => {
            // messages is not merged yet
            Ok(Responsibility::Accept)
        }
        Ok(MergeStatus::AlreadyMerged) => {
            // messages is not merged yet
            Ok(Responsibility::Accept)
        }
        Ok(MergeStatus::FreshMerge(merged_msg)) => {
            maybe_send_to_next!(&merged_msg, output_queue, channel, &msg, accept_failure)
        }
    }
}
