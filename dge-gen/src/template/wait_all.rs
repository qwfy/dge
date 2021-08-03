{% include "header.rs" %}
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    use {{ accept_failure }} as accept_failure;
    use {{ merge_messages }} as merge_messages;

    let handler_state = dge_runtime::component::wait_all::HandlerState {
        merge_messages,
        accept_failure,
        output_queue: {{ output_queue }},
    };

    let () = dge_runtime::rmq::consume_forever(
        {{ input_queue }},
        dge_runtime::component::wait_all::wait_all,
        handler_state,
        {{ prefetch_count }},
    ).await;
}
