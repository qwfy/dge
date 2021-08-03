use askama::Template;

use super::rust::gen_ident;
use super::rust::gen_opt_string;
use super::rust::gen_str;
use super::rust::gen_string;
use super::rust::gen_u32;
use super::rust::gen_vec_str;
use crate::Error;
use crate::Result;

#[derive(Template)]
#[template(path = "fan_out.rs", escape = "none")]
struct FanOutTemplate {
    type_input: String,
    accept_failure: String,
    output_queues: String,
    input_queue: String,
    prefetch_count: String,
}

pub(crate) fn generate(
    input_queue: String,
    output_queues: Vec<String>,
    accept_failure: String,
    type_input: String,
) -> Result<String> {
    let template = FanOutTemplate {
        type_input: gen_ident(type_input),
        accept_failure: gen_ident(accept_failure),
        output_queues: gen_vec_str(output_queues),
        input_queue: gen_str(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
    };

    let generated = template.render()?;

    Ok(generated)
}
