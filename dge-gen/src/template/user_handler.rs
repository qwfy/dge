{% include "header.rs" %}
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    let handler_state = dge_runtime::component::user_handler::HandlerState {
        accept_failure: {{ accept_failure }},
        output_queue: {{ output_queue }},
        user_handler: {{ user_handler }},
        user_handler_state: {{ user_handler_state }},
        
    };
    let () = dge_runtime::rmq::consumer_forever(
        {{ input_queue }},
        dge_runtime::component::user_handler::user_handler,
        handler_state,
        {{ prefetch_count }},
    );
}
