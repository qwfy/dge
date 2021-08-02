use futures::Future;
use lapin::Channel;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;

use crate::error::Error;
use crate::error::Result;
use crate::runtime::lib_rmq_primitive;
use crate::runtime::lib_rmq_primitive::Responsibility;

pub struct HandlerState<InputMsg, OutputMsg, UserError, AcceptFailureResult, UserHandlerState> {
    user_handler: fn(UserHandlerState, &InputMsg) -> std::result::Result<OutputMsg, UserError>,
    user_handler_state: UserHandlerState,
    accept_failure: fn(&InputMsg, UserError) -> AcceptFailureResult,
    output_queue: Option<String>,
}

pub async fn user_handler<
    InputMsg,
    OutputMsg,
    UserError,
    MergeResult,
    AcceptFailureResult,
    UserHandlerState,
>(
    state: HandlerState<InputMsg, OutputMsg, UserError, AcceptFailureResult, UserHandlerState>,
    channel: Channel,
    msg: InputMsg,
) -> Result<Responsibility>
where
    InputMsg: DeserializeOwned + Display + Send + 'static,
    OutputMsg: Serialize + Send + 'static,
    UserError: Display + From<serde_json::Error>,
    AcceptFailureResult: Future<Output = std::result::Result<(), UserError>>,
{
    let accept_failure = state.accept_failure;
    let output_queue = state.output_queue;
    let user_f = state.user_handler;
    let user_state = state.user_handler_state;
    match user_f(user_state, &msg) {
        Err(user_error) => {
            // TODO @incomplete: for now treat user error as final, this should be recosnidered
            warn!(
                "failed to process message: {}, error is: {}",
                &msg, &user_error
            );
            let () = accept_failure(&msg, user_error)
                .await
                .map_err(|ue| Error::UserError {
                    error: ue.to_string(),
                })?;
            Ok(Responsibility::Accept)
        }
        Ok(out_msg) => {
            maybe_send_to_next!(&out_msg, output_queue, channel, &msg, accept_failure)
        }
    }
}
