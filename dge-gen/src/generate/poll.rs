use askama::Template;

use super::graph::RmqOptions;
use super::rust::gen_ident;
use super::rust::gen_opt_str;
use super::rust::gen_str;
use super::rust::gen_u32;
use crate::Result;

#[derive(Template)]
#[template(path = "poll.rs", escape = "none")]
struct PollTemplate {
    type_input: String,
    accept_failure: String,
    output_queue: String,
    input_queue: String,
    prefetch_count: String,
    behaviour_module: String,
    rmq_options: RmqOptions,
}

pub(crate) fn generate(
    input_queue: String,
    output_queue: Option<String>,
    behaviour_module: String,
    accept_failure: String,
    type_input: String,
    rmq_options: RmqOptions,
) -> Result<String> {
    let template = PollTemplate {
        type_input: gen_ident(type_input),
        accept_failure: gen_ident(accept_failure),
        output_queue: gen_opt_str(output_queue),
        input_queue: gen_str(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
        behaviour_module: gen_ident(behaviour_module),
        rmq_options,
    };

    let generated = template.render()?;

    Ok(generated)
}
