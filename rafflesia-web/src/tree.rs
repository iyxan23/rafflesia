use std::rc::Rc;

use web_sys::MouseEvent;
use yew::{Properties, function_component, html, Html, classes, AttrValue, Callback, use_state};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Folder {
        children: Vec<Rc<Node>>
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

    pub fn new_folder(id: &str, name: &str, children: Vec<Rc<Self>>) -> Self {
        Node {
            id: id.to_string().into(),
            name: name.to_string().into(),
            kind: NodeKind::Folder { children }
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct TreeProps {
    pub root_node: Rc<Node>,
    pub click: Callback<AttrValue, ()>,

    // these callbacks gives the ID of the node
    pub new_file_click: Callback<AttrValue, ()>,
    pub new_folder_click: Callback<AttrValue, ()>,
}

#[function_component]
pub fn Tree(props: &TreeProps) -> Html {
    let collapse = use_state(|| false);
    let node = &props.root_node;

    // at this point i'm just bruteforcing how to solve this issue
    let click = Rc::new(props.click.clone());
    let new_file_click = Rc::new(props.new_file_click.clone());
    let new_folder_click = Rc::new(props.new_folder_click.clone());
    
    let onclick_id = node.id.clone();
    let onclick = if let NodeKind::File { .. } = &node.kind {
        // file
        Callback::from(move |e: MouseEvent| {
            let id = onclick_id.clone();
            click.emit(id);
        
            e.cancel_bubble();
        })
    } else {
        let collapse = collapse.clone();
        // folder, open or close
        Callback::from(move |e: MouseEvent| {
            collapse.set(!*collapse);

            e.cancel_bubble();
        })
    };

    let on_new_file_click_id = node.id.clone();
    let on_new_file_click = Callback::from(move |e: MouseEvent| {
        let id = on_new_file_click_id.clone();
        new_file_click.emit(id);

        e.cancel_bubble();
    });

    let on_new_folder_click_id = node.id.clone();
    let on_new_folder_click = Callback::from(move |e: MouseEvent| {
        let id = on_new_folder_click_id.clone();
        new_folder_click.emit(id);

        e.cancel_bubble();
    });

    let contents = if let NodeKind::Folder { children } = &node.kind {
        html! {
            <div class={classes!("children")}>
                {for children
                    .iter()
                    .map(|child| html! {
                        <Tree
                            key={child.id.as_str()}
                            click={props.click.clone()}
                            new_file_click={props.new_file_click.clone()}
                            new_folder_click={props.new_folder_click.clone()}
                            root_node={child.clone()} />
                    })}
            </div>
        }
    } else { html! {} };

    html! {
        <div class={classes!(
                "node",
                if let NodeKind::File { selected } = &node.kind {
                    if *selected { "file selected" } else { "file" }
                } else { "folder" },
                if *collapse { "collapsed" } else { "" }
            )}>
            <div class={classes!("title")}>
                <div class={classes!("text")} onclick={onclick}>
                    {node.name.clone()} if let NodeKind::Folder { .. } = &node.kind { {"/"} }   
                </div>
                <div class={classes!("actions")}>
                    if let NodeKind::Folder { .. } = &node.kind { 
                        <button onclick={on_new_file_click}>{"New file"}</button>
                        <button onclick={on_new_folder_click}>{"New folder"}</button>
                    }
                </div>
            </div>

            {contents}
        </div>
    }
}