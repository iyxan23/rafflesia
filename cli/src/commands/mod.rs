use clap::{ArgMatches, Command};
use anyhow::Result;

pub fn builtin() -> Vec<Command<'static>> {
    vec![
        new::cli(),
        compile::cli(),
        generate::cli(),
    ]
}

pub fn builtin_exec(cmd: &str) -> Option<fn(&ArgMatches) -> Result<()>> {
    Some(match cmd {
        "new" => new::exec,
        "compile" => compile::exec,
        "generate" => generate::exec,
        _ => return None
    })
}

pub mod compile;
pub mod generate;
pub mod new;