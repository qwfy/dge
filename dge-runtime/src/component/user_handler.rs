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
pub struct HandlerState<InputMsg, OutputMsg, UserError, UserHandlerState: Clone> {
    pub user_handler: fn(
        UserHandlerState,
        &InputMsg,
    ) -> Future<Output = std::result::Result<OutputMsg, UserError>>,
    pub user_handler_state: UserHandlerState,
    pub accept_failure:
        fn(&InputMsg, UserError) -> Future<Output = std::result::Result<(), UserError>>,
    pub output_queue: Option<String>,
}

pub async fn user_handler<
    InputMsg,
    OutputMsg,
    UserError,
    MergeResult,
    AcceptFailureResult,
    UserHandlerState,
    HandlerResult,
>(
    state: HandlerState<InputMsg, HandlerResult, UserError, UserHandlerState>,
    channel: Channel,
    msg: InputMsg,
) -> Result<Responsibility>
where
    InputMsg: DeserializeOwned + Display + Send + 'static,
    OutputMsg: Serialize + Send + 'static,
    UserError: Display + From<serde_json::Error>,
    AcceptFailureResult: Future<Output = std::result::Result<(), UserError>>,
    HandlerResult: Future<Output = std::result::Result<OutputMsg, UserError>>,
    UserHandlerState: Clone + Send + 'static,
{
    let accept_failure = state.accept_failure;
    let output_queue = state.output_queue;
    let user_f = state.user_handler;
    let user_state = state.user_handler_state;
    match user_f(user_state, &msg).await {
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
