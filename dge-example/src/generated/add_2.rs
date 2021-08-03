// Auto-generated, all edits will be lost on the next generation.

use dge_runtime;

// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.
#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    use crate::behaviour::accept_failure::accept_failure as accept_failure;
    use crate::behaviour::add_2::handle as user_handler;
    use crate::behaviour::add_2::init_state as init_state;

    let handler_state = dge_runtime::component::user_handler::HandlerState {
        accept_failure,
        output_queue: Some(String::from("additions")),
        user_handler,
        user_handler_state: init_state().await,

    };

    let () = dge_runtime::rmq::consume_forever(
        "input_msg_copy_2",
        dge_runtime::component::user_handler::user_handler,
        handler_state,
        1,
    ).await;
}