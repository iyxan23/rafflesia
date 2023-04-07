use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{window, EventTarget, HtmlTextAreaElement};
use yew::prelude::*;

use crate::{tree::{Tree, Node}, template::{virtfs_as_node, self}, virtfs::{Entry, VirtualFs}};

pub struct App {
    fs: VirtualFs,
    root_node: Rc<Node>,

    selected_id: Option<AttrValue>,
    selected_file_contents: AttrValue,
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
    ContentChange { event: Event }
}

impl Component for App {
    type Message = AppMessage;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let template = template::default();
        Self {
            root_node: virtfs_as_node("rafflesia-project", &template, ""),
            fs: template,

            selected_id: None,
            selected_file_contents: AttrValue::from(String::from(
                r#"
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

    - Iyxan :>"#)),
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
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("parent")}>
                <div class={classes!("left-panel")}>
                    <div class={classes!("header")}></div>
                    <Tree
                        click={ctx.link().callback(|id| AppMessage::EntryClick { id })}
                        new_file_click={ctx.link().callback(|folder| AppMessage::NewFileClick { folder })}
                        new_folder_click={ctx.link().callback(|folder| AppMessage::NewFolderClick { folder })}
                        root_node={Rc::clone(&self.root_node)} />
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
                        disabled={self.selected_id.is_none()} >
                    </textarea>
                </div>
            </div>
        }
    }
}