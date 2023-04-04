use std::ops::Deref;

use yew::{Properties, function_component, html, Html, classes, AttrValue, Callback};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: AttrValue,
    pub name: AttrValue,
    pub children: Vec<Box<Self>>
}

#[derive(Properties, PartialEq)]
pub struct TreeProps {
    pub root_node: Node,
    pub click: Callback<AttrValue, ()>
}

#[function_component]
pub fn Tree(props: &TreeProps) -> Html {
    let node = &props.root_node;
    let children = node.children.iter()
        .map(|child| html! {
            <Tree click={props.click.clone()} root_node={child.deref().clone()} />
        }).collect::<Html>();

    let onclick = Callback::from(
        move |_| todo!() /* props.click.clone().emit(node.id.clone()) */
    );

    html! {
        <div class={classes!("node")}>
            <div {onclick} class={classes!("title")}>
                {node.name.clone()}
            </div>
            <div class={classes!("children")}>
                {children}
            </div>
        </div>
    }
}