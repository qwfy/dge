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
use crate::error::Error;
use crate::error::Result;

#[derive(Template)]
#[template(path = "user_handler.rs", escape = "none")]
struct UserHandlerTemplate {
    accept_failure: String,
    output_queue: String,
    input_queue: String,
    prefetch_count: String,
    user_handler: String,
    user_handler_state: String,
}

pub(crate) fn generate(
    input_queue: String,
    output_queue: Option<String>,
    behaviour_module: String,
    accept_failure: String,
) -> Result<String> {
    let template = UserHandlerTemplate {
        accept_failure: gen_string(accept_failure),
        output_queue: gen_opt_string(output_queue),
        input_queue: gen_string(input_queue),
        // TODO @incomplete: make it configurable
        prefetch_count: gen_u32(1),
        user_handler: format!(r#"{}::handle"#, &behaviour_module),
        user_handler_state: format!(r#"{}::init_state()"#, &behaviour_module),
    };

    let generated = template.render()?;

    Ok(generated)
}
