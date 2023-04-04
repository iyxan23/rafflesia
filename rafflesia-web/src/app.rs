use yew::prelude::*;

use crate::tree::{Node, Tree};

#[function_component(App)]
pub fn app() -> Html {
    let root_node = use_state(|| Node {
        id: "root".into(),
        name: "rafflesia-project".into(),
        children: vec![]
    });

    let on_node_click = Callback::from(move |id| {
        println!("pressed: {}", id);
    });

    html! {
        <div class={classes!("parent")}>
            <div class={classes!("left-panel")}>
                <div class={classes!("header")}>
                    <button>{"New file"}</button>
                </div>
                <Tree click={on_node_click} root_node={(*root_node).clone()} />
            </div>
            <div class={classes!("code")}>
                <div class={classes!("filename")}>{"main.logic"}</div>
                <textarea></textarea>
            </div>
            <div class={classes!("under")}>
                {"Hello under"}
            </div>
        </div>
    }
}
