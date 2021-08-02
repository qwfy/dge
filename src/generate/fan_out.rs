use askama::Template;
use futures::Future;
use lapin::Channel;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Display;

use super::code_gen::gen_opt_string;
use super::code_gen::gen_string;
use super::code_gen::gen_u32;
use super::code_gen::gen_vec_string;
use crate::error::Error;
use crate::error::Result;

#[derive(Template)]
#[template(path = "fan_out.rs", escape = "none")]
struct FanOutTemplate {
    accept_failure: String,
    output_queues: String,
    input_queue: String,
    prefetch_count: String,
}

pub(crate) fn generate(
    outputs: &mut HashMap<String, String>,
    input_queue: String,
    output_queues: Vec<String>,
    accept_failure: String,
) -> Result<()> {
    let template = FanOutTemplate {
        accept_failure: gen_string(accept_failure),
        output_queues: gen_vec_string(output_queues),
        input_queue: gen_string(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
    };

    let generated = template.render()?;

    println!("{}", generated);

    Ok(())
}
