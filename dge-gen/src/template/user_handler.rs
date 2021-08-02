// Auto-generated, all edits will be lost on the next generation.

use dge;

// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.
#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub async fn main() {
    let handler_state = dge::user_handler::HandlerState {
        accept_failure: {{ accept_failure }},
        output_queue: {{ output_queue }},
        user_handler: {{ user_handler }},
        user_handler_state: {{ user_handler_state }},
        
    };
    let () = dge::runtime::lib_rmq::consumer_forever(
        {{ input_queue }},
        dge::user_handler::user_handler,
        handler_state,
        {{ prefetch_count }},
    );
}
