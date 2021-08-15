{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

use fern;
use chrono;

#[rustfmt::skip]
#[tokio::main]
pub(crate) async fn main() -> Result<()> {
    setup_logger();

    let rmq_uri = {{ rmq_options.get_rmq_uri }}();

    // all queues used in the graph
    // (work_queue, retry_queue_for_work_queue, retry_interval_in_seconds)
    let all_queues = vec![
        {%- for q in all_queues %}
        ("{{q.0}}", "{{q.1}}", {{q.2}}),
        {%- endfor %}
    ];

    let () = rmq_init::init_exchanges_and_queues(
        rmq_uri.as_ref(),
        "{{ rmq_options.work_exchange }}",
        "{{ rmq_options.retry_exchange }}",
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