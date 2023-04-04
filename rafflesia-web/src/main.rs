mod app;
mod tree;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
