{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

type HandlerState = ();

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() -> Result<()> {
    let rmq_uri = {{ rmq_options.get_rmq_uri }}();

    let handler_state = ();

    let () = dge_runtime::rmq::consume_forever(
        &rmq_uri,
        {{ input_queue }},
        handler,
        handler_state,
        {{ prefetch_count }},
    ).await;

    Ok(())
}


#[rustfmt::skip]
async fn handler(
    _state: HandlerState,
    channel: Channel,
    msg: {{ type_input }},
) -> Result<Responsibility>
{
    dge_runtime::fan_out!(
        state = state,
        channel = channel,
        msg = msg,
        accept_failure = {{ accept_failure }},
        output_queues = {{ output_queues }},
        exchange = "{{ rmq_options.work_exchange }}",
    )
}