use clap::{ArgMatches, Command};
use anyhow::Result;

pub fn builtin() -> Vec<Command<'static>> {
    vec![
        new::cli(),
        build::cli(),
        generate::cli(),
        metadata::cli(),
    ]
}

pub fn builtin_exec(cmd: &str) -> Option<fn(&ArgMatches) -> Result<()>> {
    Some(match cmd {
        "new" => new::exec,
        "build" => build::exec,
        "generate" => generate::exec,
        "metadata" => metadata::exec,
        _ => return None
    })
}

pub mod build;
pub mod generate;
pub mod new;
pub mod metadata;