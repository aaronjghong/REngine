use std::sync::Arc;

mod vk;
use vk::VkApp;

fn main() {
    let app = VkApp::new();
    app.run();
}
