use askama::Template;

use super::rust::gen_opt_string;
use super::rust::gen_string;
use super::rust::gen_u32;
use super::rust::gen_vec_string;
use crate::error::Error;
use crate::error::Result;

#[derive(Template)]
#[template(path = "wait_all.rs", escape = "none")]
struct WaitAllTemplate {
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
) -> Result<String> {
    let template = WaitAllTemplate {
        merge_messages: gen_string(merge_messages),
        accept_failure: gen_string(accept_failure),
        output_queue: gen_opt_string(output_queue),
        input_queue: gen_string(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
    };

    let generated = template.render()?;

    Ok(generated)
}
