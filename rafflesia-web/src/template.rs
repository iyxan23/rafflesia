//! A collection of templates that generates a [`VirtualFs`]
//! 
//! Templates:
//!  - [`default`]
//!    
//!    The default rafflesia template, contains files to get started with
//!    rafflesia.

use std::rc::Rc;

use crate::{virtfs::{VirtualFs, Entry}, tree::Node};

pub const TEMPLATES: [(&str, fn() -> VirtualFs); 2] = [
    ("Simple toast", default),
    ("Test template", test_template),
];

pub const DEFAULT_TEMPLATE: usize = 0;

pub fn default() -> VirtualFs {
    let vfs = VirtualFs::new(
        Entry::new_folder(
            String::from("root"),
        ).put_entry(
            String::from("swproj.toml"),
            Entry::new_file(
                String::from("swproj.toml"),
                unindent::unindent(r#"
                [project]
                name = "Rafflesia Project"               # your app name
                workspace-name = "RafflesiaProject"      # somewhat of a secondary name
                package = "com.rafflesia.project"
                version-code = 1
                version-name = "1"

                time-created = 2023-01-01T00:00:00Z
                sw-ver = 150

                [activity.main]
                logic = "main.logic"      # path relative to src/
                layout = "main.layout"

                [library.compat]
                enabled = true"#).into_bytes()
            )
        ).unwrap()
        .put_entry(
            String::from("src"),
            Entry::new_folder(String::from("src"))
                .put_entry(
                    String::from("main.logic"),
                    Entry::new_file(
                        String::from("src/main.logic"),        
                        unindent::unindent(r#"
                        number counter

                        onCreate {
                            toast("Hello rafflesia!")
                        }

                        myButton.onClick {
                            toast("Hello there!")
                        }"#).into_bytes()
                    )
                ).unwrap()
                .put_entry(
                    String::from("main.layout"),
                    Entry::new_file(
                        String::from("src/main.layout"),
                        unindent::unindent(r#"
                        LinearLayout (
                            orientation: vertical,
                            layout_width: match_parent,
                            layout_height: match_parent,
                            gravity: center
                        ) {
                            TextView (text: "Hello rafflesia"),
                            Button (text: "Click me"): myButton
                        }"#).into_bytes()
                    )
                ).unwrap()
        )
        .unwrap()
    );

    vfs
}

pub fn test_template() -> VirtualFs {
    let vfs = VirtualFs::new(
        Entry::new_folder(
            String::from("root"),
        ).put_entry(
            String::from("what is this file"),
            Entry::new_file(
                String::from("what is this file"),
                unindent::unindent(r#"alskdhalsdhkd"#).into_bytes()
            )
        ).unwrap()
    );

    vfs
}

pub fn virtfs_as_node(root_name: &str, virtfs: &VirtualFs, selected_id: &str) -> Rc<Node> {
    virtfs_entry_as_node(root_name, virtfs.get_root(), selected_id)
}

pub fn virtfs_entry_as_node(name: &str, entry: &Entry, selected_id: &str) -> Rc<Node> {
    match entry {
        Entry::File { id, .. } =>
            Rc::new(Node::new_file(id, name, selected_id == id)),
        Entry::Folder { id, children } =>
            Rc::new(Node::new_folder(
                id, name,
                children.iter()
                    .map(|(name, entry)|
                        virtfs_entry_as_node(name, entry, selected_id))
                    .collect()
                )),
    }
}