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
    setup_logger: String,
}

pub(crate) fn generate<S: AsRef<str>>(
    modules: Vec<String>,
    setup_logger: S,
) -> Result<String> {
    let setup_logger = setup_logger.as_ref();

    let mut commands = Vec::new();
    for module in modules {
        let variant = &module.to_camel_case();
        commands.push(Command {
            module,
            variant: variant.clone()
        })
    }

    let template = MainTemplate {
        commands,
        setup_logger: String::from(setup_logger),
    };

    let generated = template.render()?;

    Ok(generated)
}
