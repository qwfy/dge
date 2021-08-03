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

#[macro_export]
macro_rules! wait_all {
    (
        state=$state:ident, channel=$channel:ident, msg=$msg:ident,
        merge_messages=$merge_messages:path,
        accept_failure=$accept_failure:path,
        output_queue=$output_queue:expr $(,)?
    ) => {
        match $merge_messages(&$msg).await {
            Err(user_error) => {
                // user handler returned error,
                // since this may be a transient error (i.e. db op), we retry it.
                // this behaviour is strictly for convenience
                warn!(
                    "failed to merge messages for {}, error is: {}, will be retried",
                    &$msg, user_error
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
                $crate::maybe_send_to_next!(
                    &merged_msg,
                    $output_queue,
                    $channel,
                    &$msg,
                    $accept_failure
                )
            }
        }
    };
}
