use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};
use ariadne::{Label, Report, ReportBuilder, ReportKind, sources};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use swrs::api::block::BlockContent;
use swrs::parser::logic::BlockContainer;
use swrs::parser::Parsable;
use crate::compiler;
use crate::core::project::Project;

// todo: DI on printing things, to cover the usage of rafflesia (lib) as a library
pub fn build() -> Result<()> {
    let project = Project::find_project()?;
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

    if let Err(err) = compile(&pb, project) {
        pb.finish_and_clear();
        Err(err)
            .context(format!("Error while building project {}", project_name))?
    }

    pb.finish_and_clear();
    println!("{}", style("Done").green().to_string());

    Ok(())
}

fn compile(pb: &ProgressBar, project: Project) -> Result<()> {
    for (name, activity) in project.manifest.activity {
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

        // after we built our blocks, we'll be building the views
        let flattened = swrs::api::view::flatten_views(vec![view], None, None);

        // todo: packing

        for view in flattened {
            pb.println(view.reconstruct().unwrap());
        }

        // todo: LogicCompileResult has a lot of stuff associated with screen, this is just for
        //      demonstration and to check if the compilation is working

        for event in logic_compile_result.events {
            pb.println("\n");
            pb.println(format!(" ==> Event {}: {:?}", event.name, event.event_type));
            pb.println(" vv Reconstructed blocks vv");
            let block_container: BlockContainer = event.code.into();
            for block in block_container.0 {
                pb.println(block.reconstruct().unwrap());
            }
        }
    }

    Ok(())
}