mod app;
mod tree;
mod virtfs;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
