use std::sync::Mutex;
use std::time::Instant;
use egui::{vec2, Color32, Image, Context, TextureId};
use gl33::*;
use gl33::global_loader::*;
use glfw::{Context as OtherContext, PWindow};
use egui_glfw_gl2::GLBackEnd;

const SCREEN_WIDTH: u32 = 1600;
const SCREEN_HEIGHT: u32 = 900;
const PIC_WIDTH: i32 = 320;
const PIC_HEIGHT: i32 = 192;

mod triangle;

fn init_gl(window: &mut PWindow) {
    let window = Mutex::new(window);
    unsafe {
        load_global_gl(&|ptr| {
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            let r_str = c_str.to_str().unwrap();
            window.lock().unwrap().get_proc_address(r_str) as _
        });
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::Resizable(false));

    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Egui in GLFW!",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_all_polling(true);
    window.set_resizable(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    init_gl(&mut window);

    let mut egui_backend = GLBackEnd::new(&mut window);

    let start_time = Instant::now();
    let srgba = vec![Color32::BLACK; (PIC_HEIGHT * PIC_WIDTH) as usize];
    let plot_tex_id = egui_backend.painter.new_user_texture(
        (PIC_WIDTH as usize, PIC_HEIGHT as usize),
        &srgba,
        egui::TextureFilter::Linear,
    );

    let mut sine_shift = 0f32;
    let mut amplitude = 50f32;
    let mut test_str = "A text box to write in. Cut, copy, paste commands are available.".to_owned();

    let triangle = triangle::Triangle::new();
    let mut quit = false;

    window.set_all_polling(true);

    while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();
        egui_backend.painter.set_size(width as _, height as _);
        egui_backend.user_input.raw_input.time = Some(start_time.elapsed().as_secs_f64());
        egui_backend.egui_ctx.begin_frame(egui_backend.user_input.raw_input.take());

        unsafe {
            glClearColor(0.455, 0.302, 0.663, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
        }

        triangle.draw();

        let mut srgba: Vec<Color32> = Vec::new();
        let mut angle = 0f32;

        for y in 0..PIC_HEIGHT {
            for x in 0..PIC_WIDTH {
                srgba.push(Color32::BLACK);
                if y == PIC_HEIGHT - 1 {
                    let y = amplitude * (angle * std::f32::consts::PI / 180f32 + sine_shift).sin();
                    let y = PIC_HEIGHT as f32 / 2f32 - y;
                    srgba[(y as i32 * PIC_WIDTH + x) as usize] = Color32::YELLOW;
                    angle += 360f32 / PIC_WIDTH as f32;
                }
            }
        }
        sine_shift += 0.1f32;

        //This updates the previously initialized texture with new data.
        //If we weren't updating the texture, this call wouldn't be required.
        egui_backend.painter.update_user_texture_data(&plot_tex_id, &srgba);

        add_ui_content(&egui_backend.egui_ctx, plot_tex_id, &mut test_str, &mut amplitude, &mut quit);

        let egui_output = egui_backend.egui_ctx.end_frame();
        let platform_output = egui_output.platform_output;
        egui_backend.user_input.handle_platform_output(&egui_backend.egui_ctx, &mut window, platform_output);
        egui_backend.user_input.handle_event(&mut window, &events);

        //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        //drawing calls with it.
        //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
        let clipped_shapes = &egui_backend.egui_ctx.tessellate(egui_output.shapes, egui_backend.user_input.pixels_per_point);
        egui_backend.painter.paint_and_update_textures(egui_backend.user_input.pixels_per_point, &clipped_shapes, &egui_output.textures_delta);

        window.swap_buffers();
        glfw.poll_events();
        if quit {
            break;
        }
    }
}

pub fn add_ui_content(egui_ctx: &Context, plot_tex_id: TextureId, test_str: &mut String, amplitude: &mut f32, quit: &mut bool) {
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
        ui.add(Image::new(egui::load::SizedTexture{id: plot_tex_id, size: vec2(PIC_WIDTH as f32, PIC_HEIGHT as f32)}));
        //ui.add(Image::from_texture());
        ui.separator();
        ui.label("A simple sine wave plotted onto a GL texture then blitted to an egui managed Image.");
        ui.label(" ");
        ui.text_edit_multiline(test_str);
        ui.label(" ");
        ui.add(egui::Slider::new(amplitude, 0.0..=50.0).text("Amplitude"));
        ui.label(" ");
        if ui.button("Quit").clicked() {
            *quit = true;
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
            lorem_ipsum(ui, crate::LOREM_IPSUM_LONG);
        });
}

// ----------------------------------------------------------------------------

pub const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

pub const LOREM_IPSUM_LONG: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam various, turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis sollicitudin mauris. Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. Ut ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum. Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget, consequat quis, neque. Aliquam faucibus, elit ut dictum aliquet, felis nisl adipiscing sapien, sed malesuada diam lacus eget erat. Cras mollis scelerisque nunc. Nullam arcu. Aliquam consequat. Curabitur augue lorem, dapibus quis, laoreet et, pretium ac, nisi. Aenean magna nisl, mollis quis, molestie eu, feugiat in, orci. In hac habitasse platea dictumst.";

// ----------------------------------------------------------------------------


fn lorem_ipsum(ui: &mut egui::Ui, text: &str) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
        |ui| {
            ui.label(egui::RichText::new(text).weak());
        },
    );
}