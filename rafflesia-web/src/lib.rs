mod tree;
mod template;
mod virtfs;
mod compiler;
pub mod compiler_worker;

use std::{rc::Rc, io::Write};

use compiler_worker::{CompilerWorker, CompilerWorkerOutput, CompilerWorkerInput, ProjectData};
use gloo_dialogs::alert;
use gloo_file::{File, Blob};
use wasm_bindgen::JsCast;
use web_sys::{window, EventTarget, HtmlTextAreaElement, HtmlSelectElement};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{tree::{Tree, Node}, template::{virtfs_as_node}, virtfs::{Entry, VirtualFs}};

const WELCOME_MESSAGE: &str = r#"
   __        __   _                            _        
   \ \      / /__| | ___ ___  _ __ ___   ___  | |_ ___  
    \ \ /\ / / _ \ |/ __/ _ \| '_ ` _ \ / _ \ | __/ _ \ 
     \ V  V /  __/ | (_| (_) | | | | | |  __/ | || (_) |
      \_/\_/ \___|_|\___\___/|_| |_| |_|\___|  \__\___/ 
                                                     
 ____        __  __ _           _        __        __   _     
|  _ \ __ _ / _|/ _| | ___  ___(_) __ _  \ \      / /__| |__  
| |_) / _` | |_| |_| |/ _ \/ __| |/ _` |  \ \ /\ / / _ \ '_ \ 
|  _ < (_| |  _|  _| |  __/\__ \ | (_| |   \ V  V /  __/ |_) |
|_| \_\__,_|_| |_| |_|\___||___/_|\__,_|    \_/\_/ \___|_.__/ 

                        ---==+==---

    Welcome to Rafflesia Web! Explore the capabilities of
    my hobby lang, Rafflesia. Simply write code and
    compile them into Sketchware projects directly in your
    browser!

    Try out the examples and see what Rafflesia can do
    for you. Join our community and let's build something
    great together!

    Powered by WebAssembly, Rust and Yew.rs.

    - Iyxan :>"#;

pub struct App {
    fs: VirtualFs,
    root_node: Rc<Node>,

    selected_id: Option<AttrValue>,
    selected_file_contents: AttrValue,

    selected_template: usize,

    compiler_worker: Box<dyn Bridge<CompilerWorker>>,

    compiling: bool, // true if we're compiling, will block all user input.
    // compiled: Option<ProjectData>,
}

impl App {
    fn recreate_nodes(&mut self, selected_id: Option<&str>) {
        // fixme: unnecessary allocation
        let empty_string = AttrValue::from(String::new());

        self.root_node = virtfs_as_node(
            "rafflesia-project",
            &self.fs,
            selected_id.unwrap_or_else(||
                self.selected_id
                    .as_ref()
                    .unwrap_or(&empty_string)
            )
        );
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    EntryClick { id: AttrValue },
    NewFileClick { folder: AttrValue },
    NewFolderClick { folder: AttrValue },
    ContentChange { event: Event },
    
    ChangeTemplate { event: Event },

    CompileFinished(CompilerWorkerOutput),

    Compile,
}

impl Component for App {
    type Message = AppMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (_name, template) = template::TEMPLATES
            .get(template::DEFAULT_TEMPLATE)
            .expect("given DEFAULT_TEMPLATE doesn't exists in defined templates");

        // invoke the function
        let template = template();

        // compiler worker initialisation
        let cb = {
            let link = ctx.link().clone();
            move |e| link.send_message(Self::Message::CompileFinished(e))
        };

        let worker = CompilerWorker::bridge(Rc::new(cb));

        Self {
            root_node: virtfs_as_node("rafflesia-project", &template, ""),
            fs: template,

            selected_template: template::DEFAULT_TEMPLATE,

            selected_id: None,
            selected_file_contents: AttrValue::from(WELCOME_MESSAGE),
            compiler_worker: worker,

            compiling: false,
            // compiled: None
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let window = window().unwrap();
        match msg {
            AppMessage::EntryClick { id } => {
                // set contents to that file
                let Entry::File { content, .. } = 
                    self.fs.find_entry(id.as_str())
                        .unwrap().unwrap() else { return false; };

                self.selected_file_contents = AttrValue::from(String::from_utf8(content.clone()).unwrap());

                // recreate the nodes
                self.recreate_nodes(Some(id.as_str()));

                // open file
                self.selected_id = Some(id);

                return true;
            },
            AppMessage::NewFileClick { folder } => {
                // new file
                let Some(name) = window
                    .prompt_with_message("Specify a name:").unwrap() else {
                        window.alert_with_message("A name is required!").unwrap();
                        return false;
                    };
                
                self.fs.new_file_id(
                    folder.as_str(),
                    format!("{}/{}", folder, name),
                    name,
                    vec![]
                ).unwrap();

                self.recreate_nodes(None);

                return true;
            },
            AppMessage::NewFolderClick { folder } => {
                // new folder
                let Some(name) = window
                    .prompt_with_message("Specify a name:").unwrap() else {
                        window.alert_with_message("A name is required!").unwrap();
                        return false;
                    };
                
                self.fs.new_folder_id(
                    folder.as_str(),
                    format!("{}/{}", folder, name),
                    name,
                ).unwrap();

                self.recreate_nodes(None);

                return true;
            },
            AppMessage::ContentChange { event } => {
                let Some(selected_id) = &self.selected_id else { return false; };

                // when the user changed something in the code editor 
                let target: EventTarget = event.target().unwrap();

                // update the filesystem's file content
                let Entry::File { content, .. }
                    = self.fs.find_entry_mut(selected_id.as_str()).unwrap().unwrap() else {
                        return false;
                    };

                let value = target.unchecked_into::<HtmlTextAreaElement>().value();

                content.clear();
                content.append(&mut value.into_bytes());

                self.selected_file_contents = AttrValue::from(String::from_utf8(content.clone()).unwrap());

                return true;
            },
            AppMessage::ChangeTemplate { event } => {
                // get the selected element and retrieve which's selected
                let element = event.target().unwrap()
                    .unchecked_into::<HtmlSelectElement>();

                self.selected_template = element.selected_index() as usize;

                // completely change the fs 
                let (_name, template) = template::TEMPLATES
                    .get(self.selected_template)
                    .unwrap();

                // reset every values
                self.fs = template();
                self.selected_id = None;
                self.selected_file_contents = AttrValue::from(WELCOME_MESSAGE);
                self.recreate_nodes(None);

                return true;
            }
            AppMessage::Compile => {
                self.compiling = true;

                self.compiler_worker.send(CompilerWorkerInput { fs: self.fs.clone() });

                return true;
            },
            AppMessage::CompileFinished(response) => {
                match response {
                    CompilerWorkerOutput::Success(project_data) => {
                        // self.compiled = Some(project_data);
                        self.compiling = false;

                        // download it
                        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));

                        writer.start_file("view", Default::default()).unwrap();
                        writer.write_all(project_data.view.as_slice()).unwrap();
                        writer.start_file("logic", Default::default()).unwrap();
                        writer.write_all(project_data.logic.as_slice()).unwrap();
                        writer.start_file("file", Default::default()).unwrap();
                        writer.write_all(project_data.file.as_slice()).unwrap();
                        writer.start_file("resource", Default::default()).unwrap();
                        writer.write_all(project_data.resource.as_slice()).unwrap();
                        writer.start_file("library", Default::default()).unwrap();
                        writer.write_all(project_data.library.as_slice()).unwrap();

                        writer.add_directory("project", Default::default()).unwrap();
                        writer.start_file("project/project", Default::default()).unwrap();
                        writer.write_all(project_data.project.as_slice()).unwrap();

                        let result = writer.finish().unwrap().into_inner();

                        let blob = Blob::new_with_options(result.as_slice(), Some("application/zip"));
                        let doc = web_sys::window().unwrap()
                            .document().unwrap();
                        
                        let element = doc.create_element("a").unwrap()
                            .dyn_into::<web_sys::HtmlElement>().unwrap();

                        element.set_attribute("href", web_sys::Url::create_object_url_with_blob(&blob.into()).unwrap().as_str()).unwrap();
                        element.set_attribute("download", "compiled-project.zip").unwrap();

                        // download it :>
                        element.click();

                        return true;
                    },
                    CompilerWorkerOutput::Failure(error) => {
                        alert(&format!("Compilation failed: \n{}", error));

                        return false;
                    },
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // templates
        let templates = template::TEMPLATES.into_iter()
            .enumerate()
            .map(|(idx, (name, _func))| {
                let name = AttrValue::from(name);
                html! {
                    <option
                        selected={idx == self.selected_template}
                        key={name.as_str()}
                        value={name.clone()}>{name}</option>
                }
            });

        html! {
            <div class={classes!("parent")}>
                <div class={classes!("left-panel")}>
                    <div class={classes!("header")}>
                        {"Select a template: "}
                        <select
                            onchange={ctx.link().callback(|event| AppMessage::ChangeTemplate { event })}>
                            { for templates }
                        </select>
                    </div>
                    <Tree
                        click={ctx.link().callback(|id| AppMessage::EntryClick { id })}
                        new_file_click={ctx.link().callback(|folder| AppMessage::NewFileClick { folder })}
                        new_folder_click={ctx.link().callback(|folder| AppMessage::NewFolderClick { folder })}
                        root_node={Rc::clone(&self.root_node)} />
                    <div class={classes!("footer")}>
                        <button
                            disabled={self.compiling}
                            onclick={ctx.link().callback(|_| AppMessage::Compile)}>
                            if self.compiling {
                                {"Compiling..."}
                            } else {
                                {"Compile"}
                            }
                        </button>
                    </div>
                </div>
                <div class={classes!("code")}>
                    <div class={classes!("filename")}>
                        if let Some(name) = &self.selected_id {
                            {name}
                        } else { {"No file selected"} }
                    </div>
                    <textarea
                        wrap={"off"}
                        onchange={ctx.link().callback(|event| AppMessage::ContentChange { event })}
                        value={&self.selected_file_contents}
                        disabled={self.selected_id.is_none() || self.compiling} >
                    </textarea>
                </div>
            </div>
        }
    }
}
