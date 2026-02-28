mod app;
mod components;
mod router;

fn main() {
    // 初始化 panic hook
    console_error_panic_hook::set_once();

    // 启动 Dioxus 应用
    dioxus::launch(app::App);
}
