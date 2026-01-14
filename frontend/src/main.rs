//! RustSet Frontend - Yew 0.21 Entry Point
//! Security Asset Management Platform

use frontend::App;
use yew::Renderer;

fn main() {
    // Yew 0.21 uses Renderer::with_root to mount the app
    Renderer::<App>::with_root(gloo_utils::document().get_element_by_id("main").unwrap())
        .render();
}
