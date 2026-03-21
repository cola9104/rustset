mod app;
mod components;
mod router;
mod services;
mod state;
mod utils;
mod config;

fn main() {
    // 初始化 panic hook
    console_error_panic_hook::set_once();

    // 启动 Dioxus Web 应用
    dioxus::launch(app::App);
}
