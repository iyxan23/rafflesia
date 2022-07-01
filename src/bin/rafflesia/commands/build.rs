use clap::{AppSettings, ArgMatches, Command};
use anyhow::Result;
use rafflesia::ops::build;

pub fn cli() -> Command<'static> {
    Command::new("build")
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Builds a rafflesia project")
        .after_help("Run `rafflesia help build` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    // todo: pass args
    build::build()
}