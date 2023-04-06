use std::{ops::Deref, sync::Arc};

use yew::prelude::*;
use web_sys::window;

use crate::{tree::{Tree, Node}, template::{virtfs_as_node, self}, virtfs::{Entry, VirtualFs}};

// pub struct AppStruct {
//     fs: VirtualFs,
//     root_node: Arc<Node>,

//     selected_id: Option<AttrValue>,
//     selected_file: Option<AttrValue>,
//     selected_file_contents: Option<AttrValue>,

//     file_field_open: bool,
// }

// pub enum AppMessage {
//     EntryClick { id: AttrValue },
//     NewFileClick,
// }

// impl Component for AppStruct {
//     type Message = AppMessage;

//     type Properties = ();

//     fn create(ctx: &Context<Self>) -> Self {
//         Self {
//             fs: template::default(),
//             root_node: todo!(),
//             selected_id: todo!(),
//             selected_file: todo!(),
//             selected_file_contents: todo!(),
//             file_field_open: todo!(),
//         }
//     }

//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             AppMessage::EntryClick { id } => {

//             },
//             AppMessage::NewFileClick => {

//             },
//         }

//         true
//     }

//     fn view(&self, ctx: &Context<Self>) -> Html {
//         html! {
//             <div class={classes!("parent")}>
//                 <div class={classes!("top-part")}>
//                     <div class={classes!("left-panel")}>
//                         <div class={classes!("header")}>
//                             <button onclick={ctx.link().callback(|_| AppMessage::NewFileClick)}>{"New file"}</button>
//                         </div>
//                         <Tree
//                             new_file_click={||}
//                             new_folder_click={}
//                             click={ctx.link().callback(|id| AppMessage::EntryClick { id  })}
//                             root_node={self.root_node.clone()} />
//                     </div>
//                     <div class={classes!("code")}>
//                         <div class={classes!("filename")}>
//                             if let Some(name) = &self.selected_file {
//                                 {name}
//                             } else { {"No file selected"} }
//                         </div>
//                         <textarea value={self.selected_file_contents.clone()}></textarea>
//                     </div>
//                 </div>
//                 <div class={classes!("under")}>
//                     {"Hello under"}
//                 </div>
//             </div>
//         }
//     }
// }

#[function_component(App)]
pub fn app() -> Html {
    let bom_window = window().unwrap();
    let fs = use_state(|| template::default());
    let fs_selected_id = use_state(|| None::<String>);

    // how to make `root_node` update as `fs` gets updated as well?
    // does it already do that automatically?
    let root_node =
        use_memo(|(fs, fs_selected_id)| {
            let empty_string = String::new();
            let fs_selected_id = fs_selected_id.as_ref().unwrap_or(&empty_string);

            virtfs_as_node("rafflesia-project", &fs, &fs_selected_id)
        }, (fs.clone(), fs_selected_id.clone()));

    let selected_file = use_state(|| None::<AttrValue>);

    let on_node_click_selected_file = selected_file.clone();
    let on_node_click_selected_id = fs_selected_id.clone();
    let on_node_click = Callback::from(move |id: AttrValue| {
        // oh no
        on_node_click_selected_id.set(Some(id.to_string()));
        on_node_click_selected_file.set(Some(id));
    });

    let on_new_file_click = Callback::from(move |id: AttrValue| {
        window().unwrap()
            .alert_with_message(&format!("Creating a new file on: {}", id)).unwrap();
    });
    
    let on_new_folder_click = Callback::from(move |id: AttrValue| {
        window().unwrap()
            .alert_with_message(&format!("Creating a new folder on: {}", id)).unwrap();
    });

    let new_file_click_fs = fs.clone();
    let new_file_click = Callback::from(move |_| {
        let Some(name) = bom_window
            .prompt_with_message("Specify a filename:").unwrap() else {
                bom_window.alert_with_message("You must specify a filename!").unwrap();
                return;
            };

        let Some(path) = bom_window
            .prompt_with_message("Specify a path:").unwrap() else {
                bom_window.alert_with_message("You must specify a path!").unwrap();
                return;
            };
        
        // oh no
        let mut modified = new_file_click_fs.deref().clone();

        match modified.new_file(&path.split_terminator("/")
            .map(ToString::to_string)
            .collect::<Vec<_>>().as_slice(), path, name, vec![]
        ) {
            Ok(_) => bom_window.alert_with_message("Successful").unwrap(),
            Err(err) => bom_window.alert_with_message(&format!("Failed: {:?}", err)).unwrap(),
        }

        new_file_click_fs.set(modified);
    });

    let selected_file_contents_fs = fs.clone();
    let selected_file_contents = use_memo(
        |sf| if let Some(sf) = sf.deref() {
            // the filename
            let s = selected_file_contents_fs.clone();
            let Entry::File { content, .. } =
                    s.find_entry(sf)
                    .unwrap().unwrap()
                    else { unreachable!() };

            Some(AttrValue::from(
                String::from_utf8(content.to_vec()).expect("not utf8")
            ))
        } else { None },
        selected_file.clone()
    );

    html! {
        <div class={classes!("parent")}>
            <div class={classes!("top-part")}>
                <div class={classes!("left-panel")}>
                    <div class={classes!("header")}>
                        <button onclick={new_file_click}>{"New file"}</button>
                    </div>
                    <Tree
                        click={on_node_click}
                        new_file_click={on_new_file_click}
                        new_folder_click={on_new_folder_click}
                        root_node={root_node.deref().clone()} />
                </div>
                <div class={classes!("code")}>
                    <div class={classes!("filename")}>
                        if let Some(name) = &*selected_file.clone() {
                            {name}
                        } else { {"No file selected"} }
                    </div>
                    <textarea value={selected_file_contents.deref().clone()}></textarea>
                </div>
            </div>
            <div class={classes!("under")}>
                {"Hello under"}
            </div>
        </div>
    }
}
