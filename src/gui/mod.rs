pub use self::raw_input_translate::*;
pub use self::ui_input::*;
pub use self::ui_render::*;
pub use self::ui_context::*;

mod raw_input_translate;
mod ui_input;
mod ui_render;
mod ui_texture;
mod ui_context;

pub trait UiComponent {
    fn init(&mut self, gui_ctx: &mut GuiContext);
    fn update(&mut self, gui_ctx: &mut GuiContext);
}
