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
use dge_runtime::rmq_init;
use dge_runtime::rmq_primitive;
use dge_runtime::rmq_primitive::Responsibility;
use dge_runtime::Error;
use dge_runtime::Result;

#[rustfmt::skip]
#[tokio::main]
pub(crate) async fn main() -> Result<()> {
    let rmq_uri = dge_example::behaviour::get_rmq_uri();

    // all queues used in the graph
    // (work_queue, retry_queue_for_work_queue, retry_interval_in_seconds)
    let all_queues = vec![
        ("additions", "pre_additions_post", 13),
        ("input_msg", "pre_input_msg_post", 10),
        ("input_msg_copy_1", "pre_input_msg_copy_1_post", 11),
        ("input_msg_copy_2", "pre_input_msg_copy_2_post", 12),
        ("some_output_queue", "pre_some_output_queue_post", 1),
    ];

    let () = rmq_init::init_exchanges_and_queues(
        rmq_uri.as_ref(),
        "some_work_exchange",
        "some_retry_exchange",
        all_queues,
    ).await?;

    println!("all necessary exchanges and queues initialized");

    Ok(())
}