use anyhow::{Result, bail};
use std::collections::HashMap;

use rafflesia::{core::{self, manifest::ActivityTable}, compiler};
use swrs::{api::{screen::Screen, SketchwareProject}, parser::{file::{Orientation, KeyboardSetting, Theme}, RawSketchwareProject}};

use crate::{compiler_worker::{CompilerWorkerOutput, ProjectData}, virtfs::{VirtualFs, Entry}};

// todo: propagate errors
pub fn compile(fs: VirtualFs) -> Result<RawSketchwareProject> {
    let Some(Entry::File { content, ..  }) = fs.find_entry("swproj.toml")? else { bail!("entry not found") };
    let content = String::from_utf8(content.clone())?;

    let manifest = core::manifest::parse_manifest_str(&content)?; 

    let screens = compile_screens(
        manifest.activity.clone(), fs
    )?;

    // build a sketchware project skeleton out of the project manifest
    let mut sw_proj: SketchwareProject = manifest.try_into()?;

    // then set stuff
    sw_proj.screens = screens;

    let raw: RawSketchwareProject = sw_proj.try_into()?;

    Ok(raw)
}

fn compile_screens(activities: HashMap<String, ActivityTable>, fs: VirtualFs) -> Result<Vec<Screen>> {
    let mut screens = Vec::new();

    for (name, activity) in activities {
        // first we parse the layout
        // let layout = fs::read_to_string(
        //     Path::new("src/").join(activity.layout.as_str())
        // ).context(format!("Error while reading layout file of activity {}", name))?;
        let Some(Entry::File { content, .. }) = fs
            .find_entry(&*format!("src/{}", activity.layout.as_str()))?
            else { bail!("didn't found given layout file: `{}` of activity `{}`", activity.layout, name) };

        let layout = String::from_utf8_lossy(content);

        let parsed_layout = compiler::layout::parser::parse_layout(layout.as_ref())?;
        let view = compiler::layout::compile_view_tree(parsed_layout)?;
        

        // then parse the logic with the provided parsed layout so the logic can access views from
        // the layout (global view access baby)

        let Some(Entry::File { content, .. }) = fs
            .find_entry(&*format!("src/{}", activity.logic.as_str()))?
            else { bail!("didn't found given logic file: `{}` of activity `{}`", activity.logic, name) };

        let logic = String::from_utf8_lossy(content);

        let parsed_logic = compiler::logic::parser::parse_logic(logic.as_ref())?;
        let logic_compile_result = compiler::logic::compile_logic(parsed_logic, &view)?;

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