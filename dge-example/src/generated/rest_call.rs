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

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use dge_runtime::component::poll::Jobs;
use dge_runtime::component::poll::new_job;
use dge_runtime::component::poll::poll_forever;

#[rustfmt::skip]
#[tokio::main(worker_threads = 10)]
pub(crate) async fn main() -> Result<()> {
    // load existing jobs
    let jobs = load_jobs().await?;

    // start a thread to poll the jobs
    tokio::spawn(poll_forever(
        dge_example::behaviour::rest_call::get_capacity(),
        jobs.clone(),

        // these are used when do the actual checking
        dge_example::behaviour::rest_call::check,
        dge_example::behaviour::accept_failure::accept_failure,
        dge_example::behaviour::get_rmq_uri,
        "dge_example_work_exchange",
        Some("result"),
    ));

    let rmq_uri = dge_example::behaviour::get_rmq_uri();
    let () = dge_runtime::rmq::consume_forever(
        &rmq_uri,
        "rest_call",
        handler,
        jobs,
        1,
    ).await;

    Ok(())
}

async fn load_jobs() -> Result<Jobs<dge_example::behaviour::data::Float>> {
    let jobs = Arc::new(RwLock::new(VecDeque::new()));

    let write_jobs = jobs.clone();
    let mut write_jobs = write_jobs.write().await;

    info!("loading messages");
    let msgs = dge_example::behaviour::rest_call::init().await;

    info!("loaded {} messages, adding them to the job queue", msgs.len());
    for msg in msgs {
        let job = new_job(msg);
        write_jobs.push_back(job);
    }
    info!("jobs added to the job queue");

    Ok(jobs)
}


#[rustfmt::skip]
async fn handler(
    jobs: Jobs<dge_example::behaviour::data::Float>,
    _channel: Channel,
    msg: dge_example::behaviour::data::Float,
) -> Result<Responsibility>
{
    dge_runtime::add_to_jobs!(
        jobs = jobs,
        msg = msg,
        save_msg = dge_example::behaviour::rest_call::save_msg,
    )
}