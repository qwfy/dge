use askama::Template;
use heck::CamelCase;

use crate::Result;

struct Command {
    module: String,
    variant: String,
}

#[derive(Template)]
#[template(path = "main.rs", escape = "none")]
struct MainTemplate {
    commands: Vec<Command>,
}

pub(crate) fn generate(
    modules: Vec<String>,
) -> Result<String> {
    let mut commands = Vec::new();
    for module in modules {
        let variant = &module.to_camel_case();
        commands.push(Command {
            module,
            variant: variant.clone()
        })
    }

    let template = MainTemplate {
        commands
    };

    let generated = template.render()?;

    Ok(generated)
}
