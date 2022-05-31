use clap::{AppSettings, ArgMatches, Command};
use anyhow::Result;

pub fn cli() -> Command<'static> {
    Command::new("compile")
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Compile a rafflesia project")
        .after_help("Run `rafflesia help compile` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    println!("compile");

    Ok(())
}