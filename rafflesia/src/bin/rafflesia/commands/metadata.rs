use anyhow::{bail, Context, Result};
use clap::{AppSettings, ArgMatches, Command};
use rafflesia::core::manifest::{parse_manifest, parse_manifest_str};
use std::fs;
use std::path::Path;

pub fn cli() -> Command<'static> {
    Command::new("metadata")
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Shows the metadata of the project in the working directory")
        .after_help("Run `rafflesia help metadata` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let content = match fs::read_to_string(Path::new("swproj.toml")) {
        Ok(content) => content,
        Err(_) => bail!("Couldn't get the swproj.toml file in the current working directory"),
    };

    let manifest = parse_manifest_str(&content).context("Failed to parse swproj.toml")?;

    println!("Project name: {}", manifest.project.name);
    println!("Package: {}", manifest.project.package);

    if let Some(workspace_name) = manifest.project.workspace_name {
        println!("Workspace name: {}", workspace_name);
    }

    println!("Sketchware version: {}", manifest.project.sw_ver);
    println!("Version name: {}", manifest.project.version_name);
    println!("Version code: {}", manifest.project.version_code);

    if let Some(colors) = manifest.project.colors {
        println!("\nColors:");
        println!("  colorAccent: {}", colors.accent);
        println!("  colorPrimary: {}", colors.primary);
        println!("  colorPrimaryDark: {}", colors.primary_dark);
        println!("  colorControlHighlight: {}", colors.control_highlight);
        println!("  colorControlNormal: {}", colors.control_normal);
    }

    let activity_count = manifest.activity.len();

    print!(
        "\n{} {}",
        activity_count,
        if activity_count > 1 {
            "activities:"
        } else {
            "activity:"
        },
    );

    for (name, table) in manifest.activity {
        println!("\n - {}", name);
        println!("     logic: {}", table.logic);
        println!("     layout: {}", table.layout);
    }

    Ok(())
}
