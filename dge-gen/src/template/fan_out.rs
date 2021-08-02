{% include "header.rs" %}
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    let handler_state = dge_runtime::component::fan_out::HandlerState {
        accept_failure: {{ accept_failure }},
        output_queues: {{ output_queues }},
    };
    let () = dge_runtime::rmq::consume_forever(
        {{ input_queue }},
        dge_runtime::component::fan_out::fan_out,
        handler_state,
        {{ prefetch_count }},
    );
}
