use std::sync::Mutex;
use egui::Rgba;
use gl33::{GL_BLEND, GL_COLOR_BUFFER_BIT, GL_FRAMEBUFFER_SRGB, GL_MULTISAMPLE, GL_ONE, GL_ONE_MINUS_SRC_ALPHA};
use gl33::global_loader::{glBlendFunc, glClear, glClearColor, glEnable, load_global_gl};
use glfw::ffi::{glfwDestroyWindow, glfwSetErrorCallback, glfwTerminate};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use crate::gui::{GuiContext, UiComponent};
use crate::timer::DeltaTimer;
use crate::triangle::Triangle;

pub struct GlfwWindow {
    width: u32,
    height: u32,
    title: String,

    pub ui_contents: Vec<Box<dyn UiComponent>>
}


impl GlfwWindow {
    pub fn add_ui_component(&mut self, component: Box<dyn UiComponent>) {
        self.ui_contents.push(component);
    }

    pub fn init_ui_component(&mut self, gui_ctx: &mut GuiContext) {
        for component in &mut self.ui_contents {
            component.init(gui_ctx);
        }
    }

    pub fn update_ui_component(&mut self, gui_ctx: &mut GuiContext) {
        for component in &mut self.ui_contents {
            component.update(gui_ctx);
        }
    }
}


impl GlfwWindow {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        GlfwWindow {
            width,
            height,
            title: String::from(title),
            ui_contents: vec![],
        }
    }

    pub fn run(&mut self) {
        let (glfw, window, events) = self.init();
        let window_ptr = window.window_ptr();
        unsafe {
            self.event_loop(glfw, window, events);
            glfwSetErrorCallback(None);
            glfwDestroyWindow(window_ptr);
            glfwTerminate();
        }
    }

    fn init(&self) -> (Glfw, PWindow, GlfwReceiver<(f64, WindowEvent)>) {
        unsafe {
            // Initialize GLFW
            let mut glfw = glfw::init_no_callbacks().expect("Error: Unable to initialize GLFW.");
            glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
            glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
            glfw.window_hint(glfw::WindowHint::SRgbCapable(true));
            glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
            glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(false));
            glfw.window_hint(glfw::WindowHint::RedBits(Some(8)));
            glfw.window_hint(glfw::WindowHint::GreenBits(Some(8)));
            glfw.window_hint(glfw::WindowHint::BlueBits(Some(8)));
            glfw.window_hint(glfw::WindowHint::AlphaBits(Some(8)));
            glfw.window_hint(glfw::WindowHint::DepthBits(Some(24)));
            glfw.window_hint(glfw::WindowHint::StencilBits(Some(8)));
            glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
            glfw.window_hint(glfw::WindowHint::Resizable(true));
            // Create Window
            let (mut window, events) = glfw
                .create_window(self.width, self.height, self.title.as_str(), glfw::WindowMode::Windowed)
                .expect("Error: Failed to create GLFW window.");
            // Enable window event input
            window.set_all_polling(true);
            // Make the OpenGL context current
            window.make_current();
            // Enable v-sync
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            // Init OpenGL
            init_gl(&mut window);
            // settings
            glEnable(GL_FRAMEBUFFER_SRGB);
            glEnable(GL_MULTISAMPLE);
            glEnable(GL_BLEND);
            glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
            // Make the window visible
            window.show();
            (glfw, window, events)
        }
    }

    unsafe fn event_loop(&mut self, mut glfw: Glfw, mut window: PWindow, events: GlfwReceiver<(f64, WindowEvent)>) {
        // init timer
        let timer = DeltaTimer::new();

        // init gui
        let mut gui_ctx = GuiContext::new(&mut window);
        self.init_ui_component(&mut gui_ctx);

        let triangle = Triangle::new();

        // Set clear color
        let color = Rgba::BLACK;
        glClearColor(color.r(), color.g(), color.b(), color.a());

        // What we should do with egui in each loop:
        // 1. Update egui's data (create custom materials, update materials etc.)
        // 2. Add ui components to &egui_ctx
        // 3. Get textures to be rendered from egui_ctx, bind and upload to GPU
        // 4. Get Custom textures, bind, and upload to GPU
        // 5. Get vertex and other related data to be rendered from egui_ctx, do render
        // 6. Get materials that need to be released from egui_ctx and release them
        // PS: The material with id Manage(0) is a font, which needs to be uploaded and rendered during the first rendering and does not need to be released.

        // loop
        while !window.should_close() {
            let (width, height) = window.get_framebuffer_size();
            let pixels_per_point = window.get_content_scale().0;

            // update timer
            timer.update();

            // launch gui
            gui_ctx.start(timer.elapsed());

            // glfw poll event
            glfw.poll_events();

            // update egui
            gui_ctx.gui_render.set_size(width as _, height as _); // here because we use the "GL_SCISSOR_TEST"
            self.update_ui_component(&mut gui_ctx);
            let egui_output = gui_ctx.handle_event(&mut window, &events, pixels_per_point);

            // clear
            glClear(GL_COLOR_BUFFER_BIT);
            // draw triangle
            triangle.draw();
            // render egui
            gui_ctx.render(egui_output, pixels_per_point);
            // swap buffers
            window.swap_buffers();
        }
    }
}

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
