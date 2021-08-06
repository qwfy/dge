use askama::Template;

use super::graph::RmqOptions;
use super::rust::gen_ident;
use super::rust::gen_opt_str;
use super::rust::gen_str;
use super::rust::gen_u32;
use crate::Result;

#[derive(Template)]
#[template(path = "aggregate.rs", escape = "none")]
struct AggregateTemplate {
    type_input: String,
    aggregate: String,
    accept_failure: String,
    output_queue: String,
    input_queue: String,
    prefetch_count: String,
    rmq_options: RmqOptions,
}

pub(crate) fn generate(
    input_queue: String,
    aggregate: String,
    output_queue: Option<String>,
    accept_failure: String,
    type_input: String,
    rmq_options: RmqOptions,
) -> Result<String> {
    let template = AggregateTemplate {
        aggregate: gen_ident(aggregate),
        accept_failure: gen_ident(accept_failure),
        output_queue: gen_opt_str(output_queue),
        input_queue: gen_str(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
        type_input: gen_ident(type_input),
        rmq_options,
    };

    let generated = template.render()?;

    Ok(generated)
}
