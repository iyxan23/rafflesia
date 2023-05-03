use anyhow::Result;
use chrono::{Datelike, NaiveDate, TimeZone, Timelike, Utc};
use clap::{AppSettings, Arg, ArgMatches, Command};
use rafflesia::core::manifest::*;
use std::collections::HashMap;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use toml::value::{Date, Datetime, Offset, Time};

pub fn cli() -> Command<'static> {
    Command::new("new")
        .dont_collapse_args_in_usage(true)
        .args(&[Arg::new("folder_name")
            .help("The folder of where the project will be generated.")
            .takes_value(true)
            .required(true)])
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Create an empty rafflesia project")
        .after_help("Run `rafflesia help new` for more detailed information.\n")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let folder_name = args.value_of("folder_name").unwrap();

    println!("\n## Rafflesia project configuration ##\n");

    let name = input("App name?", "My Project")?;
    let workspace_name = input("Workspace name?", "MyProject")?;
    let package = input("Package name?", "com.my.project")?;
    let version_code = input("Version code?", "1")?.parse()?;
    let version_name = input("Version name?", "1.0")?;

    let palette = if input("Use default colors? [yes/no]", "yes")? == "no" {
        println!(" [!] Write colors as hex codes without the hash symbol (#) and transparency is optional; for instance, 008dcd and ff008dcd are the same thing.");

        let primary = input("colorPrimary:", "ff008dcd")?;
        let primary_dark = input("colorPrimaryDark:", "ff0084c2")?;
        let accent = input("colorAccent:", "ff008dcd")?;
        let control_normal = input("colorControlNormal:", "ff57beee")?;
        let control_highlight = input("colorControlHighlight:", "20008dcd")?;

        ColorsTable {
            primary,
            primary_dark,
            accent,
            control_normal,
            control_highlight,
        }
    } else {
        ColorsTable {
            primary: "ff008dcd".to_string(),
            primary_dark: "ff0084c2".to_string(),
            accent: "ff008dcd".to_string(),
            control_normal: "ff57beee".to_string(),
            control_highlight: "20008dcd".to_string(),
        }
    };

    let libraries = {
        let inp = input("Enabled libraries? [compat firebase admob googlemap]", "")?;
        let chosen_libraries: Vec<&str> = inp.split(" ").collect();

        if chosen_libraries.contains(&"firebase") || chosen_libraries.contains(&"googlemap") {
            println!(" [i] You don't have to worry too much about the id/keys for now, you can set them later in swproj.toml.");
        }

        LibraryTable {
            compat: chosen_libraries
                .contains(&"compat")
                .then(|| CompatLibraryTable { enabled: true }),

            firebase: invert::<_, anyhow::Error>(chosen_libraries.contains(&"firebase").then(
                || {
                    Ok(FirebaseLibraryTable {
                        enabled: true,
                        project_id: input(" [firebase] Project id: ", "")?,
                        app_id: input(" [firebase] App id: ", "")?,
                        api_key: input(" [firebase] Api key:", "")?,
                        storage_bucket: input(" [firebase] Storage bucket: ", "")?,
                    })
                },
            ))?,

            admob: chosen_libraries
                .contains(&"admob")
                .then(|| AdMobLibraryTable {
                    enabled: true,
                    ad_units: Default::default(), // todo
                    test_devices: vec![],         // todo
                }),

            google_map: invert::<_, anyhow::Error>(chosen_libraries.contains(&"googlemap").then(
                || {
                    Ok(GoogleMapLibraryTable {
                        enabled: true,
                        api_key: input(" [googlemap] Api key:", "")?,
                    })
                },
            ))?,
        }
    };

    // now we construct the manifest
    let mut activities = HashMap::new();
    activities.insert(
        "main".to_string(),
        ActivityTable {
            logic: "main.logic".to_string(),
            layout: "layout.logic".to_string(),
        },
    );

    let now = Utc::now();
    let manifest = toml::to_string(&Manifest {
        project: ProjectTable {
            id: None,
            name,
            workspace_name: Some(workspace_name),
            package,
            version_code,
            version_name,
            time_created: Datetime {
                date: Some(Date {
                    year: now.year() as u16,
                    month: now.month() as u8,
                    day: now.day() as u8,
                }),
                time: Some(Time {
                    hour: now.hour() as u8,
                    minute: now.minute() as u8,
                    second: now.second() as u8,
                    nanosecond: now.nanosecond(),
                }),
                // might want to add offset for better accuracy, idk tho
                offset: None,
            },
            sw_ver: 150,
            colors: Some(palette),
        },
        activity: activities,
        library: Some(libraries),
    })?;

    // we now do file operation funsies
    // create the folder and src
    fs::create_dir_all(Path::new(&format!("{}/src", folder_name)))?;

    // write the manifest
    fs::write(Path::new(&format!("{}/swrs.toml", folder_name)), manifest)?;

    // write the files and boom! we're done!
    fs::write(
        Path::new(&format!("{}/src/main.logic", folder_name)),
        include_str!("res/main_template.logic"),
    )?;

    fs::write(
        Path::new(&format!("{}/src/main.layout", folder_name)),
        include_str!("res/main_template.layout"),
    )?;

    println!("\n## Project generated into folder {}/ ##", folder_name);
    println!("## Have fun! ##\n");

    Ok(())
}

fn input(prompt: &str, default: &str) -> Result<String> {
    print!(
        "{} ({}) ",
        prompt,
        if default.len() == 0 { "empty" } else { default }
    );
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    input = input.trim().to_string();

    Ok(if input.len() == 0 {
        default.to_string()
    } else {
        input
    })
}

// flips an Option<Result<T, E>> to Result<Option<T>, E>
// credit: user.rust-lang forums, couldn't copy the link at the time, just search for "flipping
// option result to a result option rust"
fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}
