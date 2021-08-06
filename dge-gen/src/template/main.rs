{% include "part_comment.rs" %}

use structopt::StructOpt;
use dge_runtime::Result;

{% for command in commands %}
mod {{ command.module }};
{%- endfor %}

#[rustfmt::skip]
#[derive(Debug, StructOpt)]
enum Command {
    {% for command in commands %}
    {{ command.variant }},
    {%- endfor %}
}

#[rustfmt::skip]
fn main() -> Result<()> {
    let command = Command::from_args();

    match command {
        {% for command in commands %}
        Command::{{ command.variant }} => {{ command.module }}::main(),
        {%- endfor %}
    }
}