#[macro_export]
macro_rules! maybe_send_to_next {
    ($msg:expr, $queue:expr, $channel:expr, $error_context:expr, $accept_failure:path, $exchange:expr $(,)?) => {{
        // type annotation
        let queue: Option<&str> = $queue;

        match queue {
            None => {
                // no further processing needed, just accept it
                Ok(Responsibility::Accept)
            }
            Some(queue) => {
                // serialize it and send it to the output queue
                match serde_json::to_vec($msg) {
                    Err(serde_error) => {
                        // serialization error is final
                        let () = $accept_failure($error_context, serde_error.into())
                            .await
                            .map_err(|ue| Error::UserError {
                                error: ue.to_string(),
                            })?;
                        Ok(Responsibility::Accept)
                    }
                    Ok(payload) => {
                        rmq_primitive::publish($channel, $exchange, queue, payload).await?;
                        Ok(Responsibility::Accept)
                    }
                }
            }
        }
    }};
}
