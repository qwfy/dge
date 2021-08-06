#[macro_export]
macro_rules! fan_out {
    (
        state=$state:ident, channel=$channel:ident, msg=$msg:ident,
        accept_failure=$accept_failure:path,
        output_queues=$output_queues:expr,
        exchange=$exchange:expr $(,)?
    ) => {
        match serde_json::to_vec(&$msg) {
            Err(serde_error) => {
                // serialization errors are final, accept the failure
                warn!("failed to serialize message {:?}, accepting failure", &$msg);
                // failure to accept will be retried
                let () = $accept_failure((&$msg).into(), serde_error.into())
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
                for output_queue in $output_queues {
                    // TODO @incomplete: do not publish delayed
                    rmq_primitive::publish(
                        $channel.clone(),
                        $exchange,
                        output_queue,
                        payload.clone(),
                    )
                    .await?;
                }
                Ok(Responsibility::Accept)
            }
        }
    };
}
