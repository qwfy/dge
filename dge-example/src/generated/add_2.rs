// Auto-generated, all edits will be lost on the next generation.

use dge_runtime;

// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.
#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub(crate) async fn main() {
    let handler_state = dge_runtime::component::user_handler::HandlerState {
        accept_failure: crate::behaviour::accept_failure::accept_failure,
        output_queue: Some(String::from("additions")),
        user_handler: crate::behaviour::add_2::handle,
        user_handler_state: crate::behaviour::add_2::init_state(),
        
    };
    let () = dge_runtime::rmq::consume_forever(
        String::from("input_msg_copy_2"),
        dge_runtime::component::user_handler::user_handler,
        handler_state,
        1,
    );
}