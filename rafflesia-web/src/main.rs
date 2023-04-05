mod app;
mod tree;
mod template;
mod virtfs;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
