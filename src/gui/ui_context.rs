use egui::{Context, FullOutput, Pos2, RawInput, Rect, vec2};
use glfw::{GlfwReceiver, PWindow, WindowEvent};

use crate::gui::{GuiInput, GuiRender};

pub struct GuiContext {
    pub gui_render: GuiRender,
    pub egui_ctx: Context,
    pub user_input: GuiInput
}

impl GuiContext {
    pub fn new(window: &mut PWindow) -> Self {
        let pixels_per_point = window.get_content_scale().0;

        let (width, height) = window.get_framebuffer_size();
        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::new(0f32, 0f32), vec2(width as f32, height as f32) / pixels_per_point)),
            ..Default::default()
        };

        let egui_ctx = Context::default();
        egui_ctx.set_pixels_per_point(pixels_per_point);

        GuiContext {
            gui_render: GuiRender::new(width as usize, height as usize),
            egui_ctx,
            user_input: GuiInput::new(raw_input)
        }
    }
}

impl GuiContext {

    pub fn start(&mut self, elapsed_time: f64) {
        // update egui time
        self.user_input.raw_input.time = Some(elapsed_time);
        // begin egui frame
        self.egui_ctx.begin_frame(self.user_input.raw_input.take());
    }

    pub fn handle_event(&mut self, window: &mut PWindow, events: &GlfwReceiver<(f64, WindowEvent)>, pixels_per_point: f32) -> FullOutput {
        // handle egui events
        let egui_output = self.egui_ctx.end_frame();
        let platform_output = &egui_output.platform_output;
        self.user_input.handle_platform_output(window, &platform_output);
        self.user_input.handle_event(window, events, pixels_per_point);
        egui_output
    }

    pub fn render(&mut self, egui_output: FullOutput, pixels_per_point: f32) {
        // render egui
        let clipped_shapes = self.egui_ctx.tessellate(egui_output.shapes, pixels_per_point);
        self.gui_render.render(pixels_per_point, &clipped_shapes, &egui_output.textures_delta);
    }

}



