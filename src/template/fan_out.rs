// Auto-generated, all edits will be lost on the next generation.

use dge;

// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.
#[rustfmt::skip]
#[tokio::main(worker_threads = 2)]
pub async fn main() {
    let handler_state = dge::fan_out::HandlerState {
        accept_failure: {{ accept_failure }},
        output_queues: {{ output_queues }},
    };
    let () = dge::runtime::lib_rmq::consumer_forever(
        {{ input_queue }},
        dge::fan_out::fan_out,
        handler_state,
        {{ prefetch_count }},
    );
}
