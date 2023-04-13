use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{bail, Context, Result};
// use ariadne::{Label, Report, ReportBuilder, ReportKind, sources};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use swrs::api::screen::Screen;
use swrs::api::SketchwareProject;
use swrs::parser::file::{KeyboardSetting, Orientation, Theme};
use swrs::parser::RawSketchwareProject;
use crate::compiler;
use crate::core::manifest::ActivityTable;
use crate::core::project::Project;

// todo: DI on printing things, to cover the usage of rafflesia (lib) as a library
pub fn build() -> Result<()> {
    let project = Project::find_project()?;

    // verify that the project has at least one activity and at least one activity that's named "main"
    if project.manifest.activity.len() == 0 {
        bail!("There must be at least one activity");
    }

    if !project.manifest.activity.contains_key("main") {
        bail!("There must be at least one activity named `main`");
    }

    let project_name = project.manifest.project.name.to_owned();

    println!("Building project {}", style(&project_name).bold().cyan());

    let pb = ProgressBar::new_spinner()
        .with_style(ProgressStyle::default_spinner()
            .template("{spinner:.dim.bold} Building - {wide_msg}")
            .tick_chars("/|\\- ")
        );
    pb.enable_steady_tick(200);

    // start building.. i guess?
    // this is very anti-climactic

    let screens = match compile_screens(&pb, project.manifest.activity.clone()) {
        Ok(screens) => screens,
        Err(err) => {
            pb.finish_and_clear();
            return Err(err).context(format!("Error while building project {}", project_name))
        }
    };

    pb.set_message("Packing it all together");

    // build a sketchware project skeleton out of the project manifest
    let mut sw_proj: SketchwareProject = project.manifest.try_into()
        .context("Error while parsing the manifest")?;

    // then set stuff
    sw_proj.screens = screens;

    // todo: custom views
    // todo: resources

    // we just need to reconstruct this to project files

    let raw: RawSketchwareProject = sw_proj.try_into()
        .context("Error while constructing the raw sketchware project")?;

    // and we're done!!!! :D
    // now we're going to create a build folder and place our project files there
    // todo: gotta create like options to pack em into an .swb or .sh file or something

    let build_folder = project.working_directory.join("build");
    if build_folder.exists() {
        fs::remove_dir_all(&build_folder)
            .context("Failed to remove the build folder")?;
    }

    fs::create_dir(build_folder.as_path())
        .context("Failed to create the build folder")?;

    fs::write(build_folder.join("project"), swrs::encrypt_sw(raw.project.as_bytes()))
        .context("Failed to write ./build/project")?;
    fs::write(build_folder.join("file"), swrs::encrypt_sw(raw.file.as_bytes()))
        .context("Failed to write ./build/file")?;
    fs::write(build_folder.join("library"), swrs::encrypt_sw(raw.library.as_bytes()))
        .context("Failed to write ./build/library")?;
    fs::write(build_folder.join("resource"), swrs::encrypt_sw(raw.resource.as_bytes()))
        .context("Failed to write ./build/resource")?;
    fs::write(build_folder.join("view"), swrs::encrypt_sw(raw.view.as_bytes()))
        .context("Failed to write ./build/view")?;
    fs::write(build_folder.join("logic"), swrs::encrypt_sw(raw.logic.as_bytes()))
        .context("Failed to write ./build/logic")?;

    pb.println("Files written to build/");
    pb.finish_and_clear();
    println!("{}", style("Done").green().to_string());

    Ok(())
}

fn compile_screens(pb: &ProgressBar, activities: HashMap<String, ActivityTable>)
    -> Result<Vec<Screen>> {

    let mut screens = Vec::new();

    for (name, activity) in activities {
        pb.set_message(format!("Compiling {}", style(&name).cyan()));

        // todo: integrate ariadne

        // first we parse the layout
        let layout = fs::read_to_string(
            Path::new("src/").join(activity.layout.as_str())
        ).context(format!("Error while reading layout file of activity {}", name))?;

        let parsed_layout = compiler::layout::parser::parse_layout(layout.as_str())
            .context(format!("Syntax error on {}", activity.layout))?;

        let view = compiler::layout::compile_view_tree(parsed_layout)
            .context(format!("Error while compiling layout {}", activity.layout))?;

        // for later use
        // let parsed_layout = match compiler::layout::parser::parse_layout(layout.as_str()) {
        //     Ok(view) => view,
        //     Err(err) => {
        //         match err {
        //             ParseError::UnexpectedTokenError {
        //                 expected,
        //                 unexpected_token,
        //                 pos
        //             } => {
        //                 let mut builder: ReportBuilder<(String, Range<usize>)> = Report::build(ReportKind::Error, activity.layout.clone(), pos.start);
        //                 let mut label = Label::new((activity.layout.clone(), pos));
        //
        //                 if let Some(expected) = expected {
        //                     if expected.len() == 1 {
        //                         label = label.with_message(format!("Unexpected token, expected {:?}", expected.get(0).unwrap()))
        //                     } else {
        //                         label = label.with_message(
        //                             format!("Unexpected token, expected {}",
        //                             expected.into_iter()
        //                                 .fold(String::new(), |acc, tok| {
        //                                     format!("{}, {:?}", acc, tok)
        //                                 }))
        //                         );
        //                     }
        //                 } else {
        //                     label = label.with_message(format!("Unexpected token"));
        //                 }
        //
        //                 builder.add_label(label);
        //
        //                 builder.finish()
        //                     .eprint(sources(vec![(activity.layout.clone(), layout)]))
        //                     .unwrap();
        //             }
        //             ParseError::EOF { expected } => {
        //                 let end = &activity.layout.len() - 1;
        //                 Report::build(ReportKind::Error, activity.layout.clone(), end)
        //                     .with_label(Label::new((activity.layout.clone(), end..end))
        //                         .with_message(format!("EOF")))
        //                     .finish()
        //                     .eprint(sources(vec![(activity.layout.clone(), layout)]))
        //                     .unwrap();
        //             }
        //             ParseError::LexerError { err_token, pos, slice } => {
        //                 Report::build(ReportKind::Error, activity.layout.clone(), pos.start)
        //                     .with_label(Label::new((activity.layout.clone(), pos))
        //                         .with_message(format!("Invalid token: {:?}", err_token)))
        //                     .finish()
        //                     .eprint(sources(vec![(activity.layout.clone(), layout)]))
        //                     .unwrap();
        //             }
        //         }
        //         bail!("Syntax error");
        //     }
        // };

        // then parse the logic with the provided parsed layout so the logic can access views from
        // the layout (global view access baby)

        let logic = fs::read_to_string(
            Path::new("src/").join(activity.logic.as_str())
        ).context(format!("Error while reading logic file of activity {}", name))?;

        let parsed_logic = compiler::logic::parser::parse_logic(logic.as_str())
            .context(format!("Syntax error on {}", activity.logic))?;

        let logic_compile_result = compiler::logic::compile_logic(parsed_logic, &view)
            .context(format!("Error while compiling logic {}", activity.logic))?;

        screens.push(Screen {
            layout_name: name.clone(),
            java_name: view_name_to_logic(&name),

            layout: vec![view],

            variables:      logic_compile_result.variables,
            list_variables: logic_compile_result.list_variables,
            more_blocks:    logic_compile_result.more_blocks,
            components:     logic_compile_result.components,
            events:         logic_compile_result.events,

            fab: None, // todo: fab

            // todo: make these customisable on the manifest
            fullscreen_enabled: false,
            toolbar_enabled: false,
            drawer_enabled: false,
            fab_enabled: false,
            orientation: Orientation::Portrait,
            theme: Theme::None,
            keyboard_setting: KeyboardSetting::Unspecified
        });
    }

    Ok(screens)
}

// turns a view name to a logic name, something like `main` into `MainActivity`,
// `screen_display` to `ScreenDisplayActivity`
fn view_name_to_logic(s: &str) -> String {
    let mut capitalize = true;

    format!(
        "{}Activity",
        s.chars()
            .into_iter()
            .filter_map(|ch| {
                Some(if ch == '_' {
                    capitalize = true;
                    return None;
                } else if capitalize {
                    capitalize = false;
                    ch.to_ascii_uppercase()
                } else {
                    ch
                })
            })
            .collect::<String>()
    )
}
