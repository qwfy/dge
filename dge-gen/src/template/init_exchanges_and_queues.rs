{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

#[rustfmt::skip]
#[tokio::main]
pub(crate) async fn main() -> Result<()> {
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

    println!("all necessary exchanges and queues initialized");

    Ok(())
}