use egui::{Color32, TextureId, TextureOptions, vec2};
use egui_glfw_gl2::gui::{GuiContext, UiComponent};

pub struct MyUI {
    pic_width: i32,
    pic_height: i32,

    srgba: Vec<Color32>,
    plot_tex_id: Option<TextureId>,
    sine_shift: f32,
    amplitude: f32,
    test_str: String,
    quit: bool,
}

impl UiComponent for MyUI {
    fn init(&mut self, gui_ctx: &mut GuiContext) {
        self.plot_tex_id = Some(
            gui_ctx.gui_render.new_texture((self.pic_width as usize, self.pic_height as usize), &self.srgba, TextureOptions::LINEAR)
        );
    }
    fn update(&mut self, gui_ctx: &mut GuiContext) {
        let srgba = self.calc();
        gui_ctx.gui_render.update_texture(&self.plot_tex_id.unwrap(), &srgba);
        self.add_ui_content(&gui_ctx.egui_ctx)
    }
}

impl MyUI {
    pub fn new(pic_width: i32, pic_height: i32) -> Self {
        let srgba = vec![Color32::BLACK; (pic_width * pic_height) as usize];
        let mut sine_shift = 0f32;
        let mut amplitude = 50f32;
        let mut test_str = "A text box to write in. Cut, copy, paste commands are available.".to_owned();
        let mut quit = false;
        Self {
            pic_width,
            pic_height,
            srgba,
            plot_tex_id: None,
            sine_shift,
            amplitude,
            test_str,
            quit
        }
    }

    fn calc(&mut self) -> Vec<Color32> {
        let mut srgba: Vec<Color32> = Vec::new();
        let mut angle = 0f32;
        for y in 0..self.pic_height {
            for x in 0..self.pic_width {
                srgba.push(Color32::BLACK);
                if y == self.pic_height - 1 {
                    let y = self.amplitude * (angle * std::f32::consts::PI / 180f32 + self.sine_shift).sin();
                    let y = self.pic_height as f32 / 2f32 - y;
                    srgba[(y as i32 * self.pic_width + x) as usize] = Color32::YELLOW;
                    angle += 360f32 / self.pic_width as f32;
                }
            }
        }
        self.sine_shift += 0.1f32;
        srgba
    }

    fn add_ui_content(&mut self, egui_ctx: &egui::Context) {
        egui::Window::new("Egui with GLFW").resizable(true).show(&egui_ctx, |ui| {
            egui::TopBottomPanel::top("Top").show(&egui_ctx, |ui| {
                ui.menu_button("File", |ui| {
                    {
                        let _ = ui.button("test 1");
                    }
                    ui.separator();
                    {
                        let _ = ui.button("test 2");
                    }
                });
            });

            //Image just needs a texture id reference, so we just pass it the texture id that was returned to us
            //when we previously initialized the texture.
            ui.add(egui::Image::new(egui::load::SizedTexture{id: self.plot_tex_id.unwrap(), size: vec2(self.pic_width as f32, self.pic_height as f32)}));
            //ui.add(Image::from_texture());
            ui.separator();
            ui.label("A simple sine wave plotted onto a GL texture then blitted to an egui managed Image.");
            ui.label(" ");
            ui.text_edit_multiline(&mut self.test_str);
            ui.label(" ");
            ui.add(egui::Slider::new(&mut self.amplitude, 0.0..=50.0).text("Amplitude"));
            ui.label(" ");
            if ui.button("Quit").clicked() {
                let _ = &egui_ctx.set_visuals(egui::Visuals::light());
                //*quit = true;
            }
        });

        egui::Window::new("↔ freely resized")
            .vscroll(true)
            .resizable(true)
            .default_size([250.0, 150.0])
            .show(&egui_ctx, |ui| {
                ui.label("This window has empty space that fills up the available space, preventing auto-shrink.");
                ui.vertical_centered(|ui| {
                    ui.label("A simple sine wave plotted onto a GL texture then blitted to an egui managed Image.");
                });
                ui.allocate_space(ui.available_size());
            });

        egui::Window::new("↔ resizable + scroll")
            .vscroll(true)
            .resizable(true)
            .default_height(300.0)
            .show(&egui_ctx, |ui| {
                ui.label(
                    "This window is resizable and has a scroll area. You can shrink it to any size.",
                );
                ui.separator();
                lorem_ipsum(ui, LOREM_IPSUM_LONG);
            });
    }

}

// ----------------------------------------------------------------------------

pub const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
pub const LOREM_IPSUM_LONG: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam various, turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis sollicitudin mauris. Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. Ut ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum. Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget, consequat quis, neque. Aliquam faucibus, elit ut dictum aliquet, felis nisl adipiscing sapien, sed malesuada diam lacus eget erat. Cras mollis scelerisque nunc. Nullam arcu. Aliquam consequat. Curabitur augue lorem, dapibus quis, laoreet et, pretium ac, nisi. Aenean magna nisl, mollis quis, molestie eu, feugiat in, orci. In hac habitasse platea dictumst.";

fn lorem_ipsum(ui: &mut egui::Ui, text: &str) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
        |ui| {
            ui.label(egui::RichText::new(text).weak());
        },
    );
}