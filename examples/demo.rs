use egui_glfw_gl2::glfw_window::GlfwWindow;
use crate::data::myui::MyUI;

mod data;

fn main() {
    let mut window = GlfwWindow::new(1280, 720, "test");
    window.add_ui_component(Box::new(MyUI::new(320, 192)));
    window.run();
}