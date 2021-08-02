macro_rules! maybe_send_to_next {
    ($msg:expr, $queue:expr, $channel:expr, $accept_failure_msg:expr, $accept_failure:ident) => {{
        use log::{debug, info, warn};
        use serde::Serialize;
        use serde_json;
        use std::fmt::Display;

        use crate::error::Error;
        use crate::error::Result;
        use crate::runtime::lib_rmq_primitive;
        use crate::runtime::lib_rmq_primitive::Responsibility;

        match $queue {
            None => {
                // no further processing needed, just accept it
                Ok(Responsibility::Accept)
            }
            Some(queue) => {
                // serialize it and send it to the output queue
                match serde_json::to_vec($msg) {
                    Err(serde_error) => {
                        // serialization error is final
                        let () = $accept_failure($accept_failure_msg, serde_error.into())
                            .await
                            .map_err(|ue| Error::UserError {
                                error: ue.to_string(),
                            })?;
                        Ok(Responsibility::Accept)
                    }
                    Ok(payload) => {
                        lib_rmq_primitive::publish_delayed(Some($channel), &queue, payload).await?;
                        Ok(Responsibility::Accept)
                    }
                }
            }
        }
    }};
}
