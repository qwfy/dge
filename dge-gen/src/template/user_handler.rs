{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() -> Result<()> {
    let rmq_uri = {{ rmq_options.get_rmq_uri }}();

    let handler_state = {{ behaviour_module }}::init().await;

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
    state: {{ behaviour_module }}::State,
    channel: Channel,
    msg: {{ type_input }},
) -> Result<Responsibility>
{
    dge_runtime::user_handler!(
        state = state,
        channel = channel,
        msg = msg,
        user_handler = {{ behaviour_module }}::handle,
        accept_failure = {{ accept_failure }},
        output_queue = {{ output_queue }},
        exchange = "{{ rmq_options.work_exchange }}",
    )
}