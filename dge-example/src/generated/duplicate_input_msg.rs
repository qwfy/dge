// Auto-generated, all edits will be lost on the next generation.
//
// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.

use lapin::Channel;
use log::debug;
use log::info;
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::fmt::Display;

use dge_runtime;
use dge_runtime::rmq_primitive;
use dge_runtime::rmq_primitive::Responsibility;
use dge_runtime::Error;
use dge_runtime::Result;

type HandlerState = ();

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    let handler_state = ();

    let () = dge_runtime::rmq::consume_forever(
        "input_msg",
        handler,
        handler_state,
        1,
    ).await;
}


#[rustfmt::skip]
async fn handler(
    _state: HandlerState,
    channel: Channel,
    msg: i32,
) -> Result<Responsibility>
{
    dge_runtime::fan_out!(
        state = state,
        channel = channel,
        msg = msg,
        accept_failure = crate::behaviour::accept_failure::accept_failure,
        output_queues = vec!["input_msg_copy_2","input_msg_copy_1"],
    )
}