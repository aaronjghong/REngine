use std::sync::Arc;

mod vk;
use vk::VkApp;

fn main() {
    let mut app = VkApp::new();
    app.triangle_sample();
    app.run();
}
