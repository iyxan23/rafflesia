use anyhow::Result;
use clap::{AppSettings, ArgMatches, Command};

pub fn cli() -> Command<'static> {
    Command::new("generate")
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Generate a rafflesia project from a sketchware project")
        .after_help("Run `rafflesia help generate` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    println!("Generating rafflesia projects is currently work-in-progress");

    Ok(())
}
