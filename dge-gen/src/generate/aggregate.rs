use askama::Template;

use super::rust::gen_ident;
use super::rust::gen_opt_string;
use super::rust::gen_str;
use super::rust::gen_string;
use super::rust::gen_u32;
use super::rust::gen_vec_string;
use crate::Error;
use crate::Result;

#[derive(Template)]
#[template(path = "aggregate.rs", escape = "none")]
struct WaitAllTemplate {
    type_input_msg: String,
    type_error: String,
    merge_messages: String,
    accept_failure: String,
    output_queue: String,
    input_queue: String,
    prefetch_count: String,
}

pub(crate) fn generate(
    input_queue: String,
    merge_messages: String,
    output_queue: Option<String>,
    accept_failure: String,
    type_input_msg: String,
    type_error: String,
) -> Result<String> {
    let template = WaitAllTemplate {
        merge_messages: gen_ident(merge_messages),
        accept_failure: gen_ident(accept_failure),
        output_queue: gen_opt_string(output_queue),
        input_queue: gen_str(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
        type_input_msg: gen_ident(type_input_msg),
        type_error: gen_ident(type_error),
    };

    let generated = template.render()?;

    Ok(generated)
}