{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use dge_runtime::component::poll::Jobs;
use dge_runtime::component::poll::new_job;
use dge_runtime::component::poll::add_to_jobs;
use dge_runtime::component::poll::poll_forever;

#[rustfmt::skip]
#[tokio::main(worker_threads = 10)]
pub(crate) async fn main() -> Result<()> {
    // load existing jobs
    let jobs = load_jobs().await?;

    // start a thread to poll the jobs
    tokio::spawn(poll_forever(
        {{ behaviour_module }}::get_capacity(),
        jobs.clone(),

        // these are used when do the actual checking
        {{ behaviour_module }}::check,
        {{ accept_failure }},
        {{ rmq_options.get_rmq_uri }},
        "{{ rmq_options.work_exchange }}",
        {{ output_queue }},
    ));

    let rmq_uri = {{ rmq_options.get_rmq_uri }}();
    let () = dge_runtime::rmq::consume_forever(
        &rmq_uri,
        {{ input_queue }},
        add_to_jobs,
        jobs,
        {{ prefetch_count }},
    ).await;

    Ok(())
}

async fn load_jobs() -> Result<Jobs<{{ type_input }}>> {
    let jobs = Arc::new(RwLock::new(VecDeque::new()));

    let write_jobs = jobs.clone();
    let mut write_jobs = write_jobs.write().await;

    info!("loading messages");
    let msgs = {{ behaviour_module }}::init().await;

    info!("loaded {} messages, adding them to the job queue", msgs.len());
    for msg in msgs {
        let job = new_job(msg);
        write_jobs.push_back(job);
    }
    info!("jobs added to the job queue");

    Ok(jobs)
}