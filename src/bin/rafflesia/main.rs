use std::process::{ExitCode, Termination};
use anyhow::{Result, Context};
use clap::{AppSettings, Command};
use console::style;

mod commands;

fn try_main() -> Result<()> {
    let args = Command::new("rafflesia")
        .about("A simple language for sketchware projects")
        .setting(AppSettings::DeriveDisplayOrder)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommands(commands::builtin())
        .get_matches();

    if let Some((subcommand, args)) = args.subcommand() {
        let func = commands::builtin_exec(subcommand)
            .context(format!("Subcommand {} does not exist", subcommand))?;

        func(args)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = try_main() {
        eprintln!("\n{} {}", style("error:").bold().red(), err);
        eprintln!();
        eprintln!("Caused by:");
        err.chain().skip(1)
            .for_each(|err_item| {
                eprintln!("    {}", err_item);
            });
        eprintln!();

        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}