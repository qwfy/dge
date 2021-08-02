use futures::Future;
use lapin::Channel;
use log::{debug, info, warn};
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;

use crate::error::Error;
use crate::error::Result;
use crate::primitive::lib_rmq_primitive;
use crate::primitive::lib_rmq_primitive::Responsibility;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) fn generate(
    outputs: &mut HashMap<String, String>,
    input_queue: String,
    merge_messages: String,
    output_queue: Option<String>,
) -> Result<()> {
    unimplemented!()
}

/// Status of merging multiple messages into one `MergedMsg`
pub enum MergeStatus<MergedMsg> {
    /// The merge is incomplete, need more incoming messages.
    Partial,
    /// For the first time, the multiple input messages are merged into one.
    FreshMerge(MergedMsg),
    /// The incoming messages are already merged previously.
    AlreadyMerged,
}

type HandlerState = ();

pub async fn wait_all<InputMsg, MergedMsg, UserError, MergeResult, AcceptFailureResult>(
    state: HandlerState,
    channel: Channel,
    msg: InputMsg,
    merge_messages: fn(&InputMsg) -> MergeResult,
    accept_failure: fn(&InputMsg, UserError) -> AcceptFailureResult,
    output_queue: Option<String>,
) -> Result<Responsibility>
where
    InputMsg: DeserializeOwned + Display + Send + 'static,
    MergedMsg: Serialize + Send + 'static,
    UserError: Display + From<serde_json::Error>,
    MergeResult: Future<Output = std::result::Result<MergeStatus<MergedMsg>, UserError>>,
    AcceptFailureResult: Future<Output = std::result::Result<(), UserError>>,
{
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
            match output_queue {
                None => {
                    // no further processing needed, just accept it
                    Ok(Responsibility::Accept)
                }
                Some(queue) => {
                    // serialize it and send it to the output queue
                    match serde_json::to_vec(&merged_msg) {
                        Err(serde_error) => {
                            // serialization error is final
                            let () =
                                accept_failure(&msg, serde_error.into())
                                    .await
                                    .map_err(|ue| Error::UserError {
                                        error: ue.to_string(),
                                    })?;
                            Ok(Responsibility::Accept)
                        }
                        Ok(payload) => {
                            lib_rmq_primitive::publish_delayed(Some(channel), &queue, payload)
                                .await?;
                            Ok(Responsibility::Accept)
                        }
                    }
                }
            }
        }
    }
}
