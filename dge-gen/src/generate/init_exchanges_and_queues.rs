use askama::Template;

use super::graph::RmqOptions;
use crate::Error;
use crate::Result;


#[derive(Template)]
#[template(path = "init_exchanges_and_queues.rs", escape = "none")]
struct DeclareTemplate {
    rmq_options: RmqOptions,
    all_queues: Vec<(String, String, u32)>
}

pub(crate) fn generate(
    rmq_options: RmqOptions,
    all_queues: Vec<(String, String, u32)>,
) -> Result<String> {
    let template = DeclareTemplate {
        rmq_options,
        all_queues,
    };

    let generated = template.render()?;

    Ok(generated)
}
