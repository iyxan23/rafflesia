use anyhow::{Result, Context};
use clap::{AppSettings, Command};

mod commands;

fn main() -> Result<()> {
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
