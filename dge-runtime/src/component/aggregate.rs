/// Status of merging multiple messages into one `Aggregated`
pub enum AggregationStatus<Aggregated> {
    /// The merge is incomplete, need more incoming messages.
    Ignore,
    /// For the first time, the multiple input messages are merged into one.
    Aggregated(Aggregated),
}

#[macro_export]
macro_rules! aggregate {
    (
        state=$state:ident, channel=$channel:ident, msg=$msg:ident,
        aggregate=$aggregate:path,
        accept_failure=$accept_failure:path,
        output_queue=$output_queue:expr,
        exchange=$exchange:expr $(,)?
    ) => {
        match $aggregate($state, &$msg).await {
            Err(user_error) => {
                // user handler returned error,
                // since this may be a transient error (i.e. db op), we retry it.
                // this behaviour is strictly for convenience
                warn!(
                    "failed to merge messages for {:?}, error is: {}, will be retried",
                    &$msg, user_error
                );
                // reject the message to retry
                Ok(Responsibility::Reject)
            }
            Ok(AggregationStatus::Ignore) => {
                Ok(Responsibility::Accept)
            }
            Ok(AggregationStatus::Aggregated(merged_msg)) => {
                $crate::maybe_send_to_next!(
                    &merged_msg,
                    $output_queue,
                    $channel,
                    $msg.into(),
                    $accept_failure,
                    $exchange,
                )
            }
        }
    };
}
