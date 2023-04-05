use std::{cell::RefCell, sync::Arc};

use yew::prelude::*;
use web_sys::{console, window};

use crate::{tree::Tree, template::{virtfs_as_node, self}};

// Yew.rs feels like I can just throw any terribly unoptimized code at it
// and make it seem like nothing. I don't know if doing this is a bad thing
// there doesn't seem to be any other way of doing this.

pub fn app() -> Html {
    let fs = use_state(||
        RefCell::new(template::default())
    );

    // how to make `root_node` update as `fs` gets updated as well?
    // does it already do that automatically?
    let root_node_fs = fs.clone();
    let root_node =
        use_memo(|fs|
            virtfs_as_node("rafflesia-project", root_node_fs),
            fs.clone()
        );

    let selected_file = use_state(|| None::<AttrValue>);

    let on_node_click = Callback::from(move |id: AttrValue| {
        console::log_1(&format!("id: {}", id.as_str()).into());
    });

    let new_file_click_fs = fs.clone();
    let new_file_click = Callback::from(move |_| {
        let window = window().unwrap();
        let Some(name) = window
            .prompt_with_message("Specify a filename:").unwrap() else {
                window.alert_with_message("You must specify a filename!").unwrap();
                return;
            };

        let Some(path) = window
            .prompt_with_message("Specify a path:").unwrap() else {
                window.alert_with_message("You must specify a path!").unwrap();
                return;
            };
        
        match new_file_click_fs
            .borrow_mut()
            .new_file(&path.split("/")
                .map(ToString::to_string)
                .collect::<Vec<_>>().as_slice(), path, name, vec![]
            ) {
            Ok(_) => window.alert_with_message("Success").unwrap(),
            Err(err) => window.alert_with_message(&*format!("Error: {:?}", err)).unwrap(),
        }
    });

    html! {
        <div class={classes!("parent")}>
            <div class={classes!("top-part")}>
                <div class={classes!("left-panel")}>
                    <div class={classes!("header")}>
                        <button onclick={new_file_click}>{"New file"}</button>
                    </div>
                    <Tree click={on_node_click} root_node={root_node.clone()} />
                </div>
                <div class={classes!("code")}>
                    <div class={classes!("filename")}>
                        if let Some(selected_file) = &*selected_file.clone() {
                            {selected_file}
                        } else { {"No file selected"} }
                    </div>
                    <textarea></textarea>
                </div>
            </div>
            <div class={classes!("under")}>
                {"Hello under"}
            </div>
        </div>
    }
}
