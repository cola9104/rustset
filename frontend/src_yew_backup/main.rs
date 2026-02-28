//! RustSet Frontend - Yew 0.21 Entry Point
//! Security Asset Management Platform

use frontend::App;
use yew::Renderer;
use gloo_console::log;

fn main() {
    // Set panic hook for better error messages in console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    log!("Starting RustSet Frontend...");
    
    // Yew 0.21 uses Renderer::with_root to mount the app
    let document = gloo_utils::document();
    let mount_point = document.get_element_by_id("main").expect("Failed to find #main element");
    
    log!("Found mount point, rendering app...");
    
    Renderer::<App>::with_root(mount_point).render();
}
