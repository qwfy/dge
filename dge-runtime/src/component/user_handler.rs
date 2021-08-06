#[macro_export]
macro_rules! user_handler {
    (
        state=$state:ident, channel=$channel:ident, msg=$msg:ident,
        user_handler=$user_handler:path,
        accept_failure=$accept_failure:path,
        output_queue=$output_queue:expr,
        exchange=$exchange:expr $(,)?
    ) => {
        match $user_handler($state, &$msg).await {
            Err(user_error) => {
                // TODO @incomplete: for now treat user error as final, this should be recosnidered
                warn!(
                    "failed to process message: {:?}, error is: {}",
                    &$msg, &user_error
                );
                let () =
                    $accept_failure((&$msg).into(), user_error)
                        .await
                        .map_err(|ue| Error::UserError {
                            error: ue.to_string(),
                        })?;
                Ok(Responsibility::Accept)
            }
            Ok(out_msg) => {
                $crate::maybe_send_to_next!(
                    &out_msg,
                    $output_queue,
                    $channel,
                    (&$msg).into(),
                    $accept_failure,
                    $exchange,
                )
            }
        }
    };
}
