use std::sync::Arc;

use yew::{Properties, function_component, html, Html, classes, AttrValue, Callback, use_state};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Folder {
        children: Vec<Arc<Node>>
    },
    File {
        selected: bool
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: AttrValue,
    pub name: AttrValue,
    pub kind: NodeKind
}

impl Node {
    pub fn new_file(id: &str, name: &str, selected: bool) -> Self {
        Node {
            id: id.to_string().into(),
            name: name.to_string().into(),
            kind: NodeKind::File { selected },
        }
    }

    pub fn new_folder(id: &str, name: &str, children: Vec<Arc<Self>>) -> Self {
        Node {
            id: id.to_string().into(),
            name: name.to_string().into(),
            kind: NodeKind::Folder { children }
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct TreeProps {
    pub root_node: Arc<Node>,
    pub click: Callback<AttrValue, ()>
}

#[function_component]
pub fn Tree(props: &TreeProps) -> Html {
    let collapse = use_state(|| false);
    let node = &props.root_node;

    // at this point i'm just bruteforcing how to solve this issue
    let click = Arc::new(props.click.clone());
    let id = node.id.clone();

    let onclick = if let NodeKind::File { .. } = &node.kind {
        // file
        Callback::from(move |_| {
            let id = id.clone();
            click.emit(id);
        })
    } else {
        let collapse = collapse.clone();
        // folder, open or close
        Callback::from(move |_| collapse.set(!*collapse))
    };

    let contents = if let NodeKind::Folder { children } = &node.kind {
        html! {
            <div class={classes!("children")}>
                {children
                    .iter()
                    .map(|child| html! {
                        <Tree click={props.click.clone()} root_node={child.clone()} />
                    }).collect::<Html>()}
            </div>
        }
    } else { html! {} };

    html! {
        <div class={classes!("node")}>
            <div onclick={onclick} class={classes!("title", if let NodeKind::File { selected } = &node.kind {
                if *selected { "selected" }
                else { "" }
            } else { ""})}>
                {node.name.clone()} if let NodeKind::Folder { .. } = &node.kind { {"/"} }
            </div>

            if !*collapse {
                {contents}
            }
        </div>
    }
}