{% include "header_comment.rs" %}

{% include "header_common_import.rs" %}

#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    use {{ accept_failure }} as accept_failure;
    use {{ behaviour_module }}::handle as user_handler;
    use {{ behaviour_module }}::init_state as init_state;

    let handler_state = dge_runtime::component::user_handler::HandlerState {
        accept_failure,
        output_queue: {{ output_queue }},
        user_handler,
        user_handler_state: init_state().await,

    };

    let () = dge_runtime::rmq::consume_forever(
        {{ input_queue }},
        dge_runtime::component::user_handler::user_handler,
        handler_state,
        {{ prefetch_count }},
    ).await;
}
