{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

use dge_runtime::component::aggregate::AggregationStatus;

type HandlerState = ();

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    let handler_state = ();

    let () = dge_runtime::rmq::consume_forever(
        {{ input_queue }},
        handler,
        handler_state,
        {{ prefetch_count }},
    ).await;
}


#[rustfmt::skip]
async fn handler(
    _state: HandlerState,
    channel: Channel,
    msg: {{ type_input }},
) -> Result<Responsibility>
{
    dge_runtime::aggregate!(
        state = state,
        channel = channel,
        msg = msg,
        aggregate = {{ aggregate }},
        accept_failure = {{ accept_failure }},
        output_queue = {{ output_queue }},
    )
}