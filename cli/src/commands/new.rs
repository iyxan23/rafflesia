use clap::{AppSettings, ArgMatches, Command};
use anyhow::Result;

pub fn cli() -> Command<'static> {
    Command::new("new")
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Create an empty rafflesia project")
        .after_help("Run `rafflesia help new` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    println!("new");

    Ok(())
}