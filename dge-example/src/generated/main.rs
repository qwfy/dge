// Auto-generated, all edits will be lost on the next generation.
//
// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.

use structopt::StructOpt;
use dge_runtime::Result;

// these are the codes for each node
mod double;
mod duplicate_input;
mod init_exchanges_and_queues;
mod multiply;
mod square;

#[rustfmt::skip]
#[derive(Debug, StructOpt)]
enum Command {
    Double,
    DuplicateInput,
    InitExchangesAndQueues,
    Multiply,
    Square,
}

#[rustfmt::skip]
fn main() -> Result<()> {
    let command = Command::from_args();

    match command {
        Command::Double => double::main(),
        Command::DuplicateInput => duplicate_input::main(),
        Command::InitExchangesAndQueues => init_exchanges_and_queues::main(),
        Command::Multiply => multiply::main(),
        Command::Square => square::main(),
    }
}