// Auto-generated, all edits will be lost on the next generation.
//
// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.

use structopt::StructOpt;
use dge_runtime::Result;

// these are the codes for each node
mod add_1;
mod add_2;
mod duplicate_input;
mod init_exchanges_and_queues;
mod wait_additions;

#[rustfmt::skip]
#[derive(Debug, StructOpt)]
enum Command {
    Add1,
    Add2,
    DuplicateInput,
    InitExchangesAndQueues,
    WaitAdditions,
}

#[rustfmt::skip]
fn main() -> Result<()> {
    let command = Command::from_args();

    match command {
        Command::Add1 => add_1::main(),
        Command::Add2 => add_2::main(),
        Command::DuplicateInput => duplicate_input::main(),
        Command::InitExchangesAndQueues => init_exchanges_and_queues::main(),
        Command::WaitAdditions => wait_additions::main(),
    }
}