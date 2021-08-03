{% include "part_comment.rs" %}

{% include "part_common_import.rs" %}

use dge_runtime::component::aggregate::MergeStatus;

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
    msg: {{ type_input_msg }},
) -> Result<Responsibility>
{
    dge_runtime::aggregate!(
        state = state,
        channel = channel,
        msg = msg,
        merge_messages = {{ merge_messages }},
        accept_failure = {{ accept_failure }},
        output_queue = {{ output_queue }},
    )
}