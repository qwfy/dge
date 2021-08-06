// Auto-generated, all edits will be lost on the next generation.
//
// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.

#[allow(unused_imports)]
use lapin::Channel;
use log::debug;
use log::info;
use log::warn;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

#[allow(unused_imports)]
use dge_runtime;
use dge_runtime::rmq_init;
use dge_runtime::rmq_primitive;
use dge_runtime::rmq_primitive::Responsibility;
use dge_runtime::Error;
use dge_runtime::Result;

type HandlerState = ();

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() -> Result<()> {
    let rmq_uri = dge_example::behaviour::get_rmq_uri();

    let handler_state = ();

    let () = dge_runtime::rmq::consume_forever(
        &rmq_uri,
        "input",
        handler,
        handler_state,
        1,
    ).await;

    Ok(())
}


#[rustfmt::skip]
async fn handler(
    _state: HandlerState,
    channel: Channel,
    msg: dge_example::behaviour::data::Integer,
) -> Result<Responsibility>
{
    dge_runtime::fan_out!(
        state = state,
        channel = channel,
        msg = msg,
        accept_failure = dge_example::behaviour::accept_failure::accept_failure,
        output_queues = vec!["input_copy_2","input_copy_1"],
        exchange = "dge_example_work_exchange",
    )
}