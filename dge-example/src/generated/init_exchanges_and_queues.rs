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

use fern;
use chrono;

#[rustfmt::skip]
#[tokio::main]
pub(crate) async fn main() -> Result<()> {
    setup_logger();

    let rmq_uri = dge_example::behaviour::get_rmq_uri();

    // all queues used in the graph
    // (work_queue, retry_queue_for_work_queue, retry_interval_in_seconds)
    let all_queues = vec![
        ("input", "retry_input", 10),
        ("input_copy_1", "retry_input_copy_1", 11),
        ("input_copy_2", "retry_input_copy_2", 12),
        ("multiply", "retry_multiply", 13),
        ("rest_call", "retry_rest_call", 13),
        ("result", "retry_result", 1),
    ];

    let () = rmq_init::init_exchanges_and_queues(
        rmq_uri.as_ref(),
        "dge_example_work_exchange",
        "dge_example_retry_exchange",
        all_queues,
    ).await?;

    info!("all necessary exchanges and queues initialized");

    Ok(())
}

#[rustfmt::skip]
fn setup_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d] [%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}