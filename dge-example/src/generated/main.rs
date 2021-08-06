// Auto-generated, all edits will be lost on the next generation.
//
// Rustfmt is disabled.
// This is to keep the style of generated content consistent between multiple generation runs,
// so that there is no unnecessary VCS diff to review
// even if the toolchain formats this file automatically.

use structopt::StructOpt;
use dge_runtime::Result;


mod wait_additions;
mod init_exchanges_and_queues;
mod add_2;
mod add_1;
mod duplicate_input_msg;

#[rustfmt::skip]
#[derive(Debug, StructOpt)]
enum Command {
    
    WaitAdditions,
    InitExchangesAndQueues,
    Add2,
    Add1,
    DuplicateInputMsg,
}

#[rustfmt::skip]
fn main() -> Result<()> {
    let command = Command::from_args();

    match command {
        
        Command::WaitAdditions => wait_additions::main(),
        Command::InitExchangesAndQueues => init_exchanges_and_queues::main(),
        Command::Add2 => add_2::main(),
        Command::Add1 => add_1::main(),
        Command::DuplicateInputMsg => duplicate_input_msg::main(),
    }
}