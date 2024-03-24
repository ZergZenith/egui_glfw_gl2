mod input_translate;
mod painter;
mod egui_shader;

use std::ptr;
use clipboard::{ClipboardContext, ClipboardProvider};
use egui::{Context, CursorIcon, Event, Modifiers, PlatformOutput, Pos2, pos2, RawInput, Rect, vec2};
use glfw::{GlfwReceiver, PWindow, WindowEvent};
use winapi::um::winuser;
use crate::input_translate::{is_copy_command, is_cut_command, is_paste_command, translate_cursor, translate_modifiers, translate_virtual_key_code};
use crate::painter::Painter;

pub struct GLBackEnd {
    pub painter: Painter,
    pub egui_ctx: Context,
    pub user_input: UserInputState
}

impl GLBackEnd {
    pub fn new(window: &mut PWindow) -> Self {
        let pixels_per_point = window.get_content_scale().0;
        
        let (width, height) = window.get_framebuffer_size();
        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::new(0f32, 0f32),
                vec2(width as f32, height as f32) / pixels_per_point
            )),
            ..Default::default()
        };
        
        let egui_ctx = Context::default();
        egui_ctx.set_pixels_per_point(pixels_per_point);
        
        GLBackEnd {
            painter: Painter::new(window),
            egui_ctx,
            user_input: UserInputState::new(raw_input, pixels_per_point)
        }
    }
}

pub struct UserInputState {
    pub raw_input: RawInput,
    pub pixels_per_point: f32,
    pub clipboard: Option<ClipboardContext>,
    pub modifiers: Modifiers,
    pub focus: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub cursor_pos: Pos2,
    pub cursor_in_window: bool,
    pub cursor_current_icon: CursorIcon,
}

impl UserInputState {
    pub fn new(input: RawInput, pixels_per_point: f32) -> Self {
        let clipboard = match ClipboardContext::new() {
            Ok(clipboard) => Some(clipboard),
            Err(err) => {
                eprintln!("Failed to initialize clipboard: {}", err);
                None
            }
        };
        UserInputState {
            raw_input: input,
            pixels_per_point,
            clipboard,
            modifiers: Modifiers::default(),
            focus: true,
            minimized: true,
            maximized: false,
            cursor_pos: Pos2::new(0f32, 0f32),
            cursor_in_window: false,
            cursor_current_icon: CursorIcon::None,
        }
    }
    
    pub fn handle_platform_output(&mut self, egui_ctx: &Context, window: &mut PWindow, platform_output: PlatformOutput) {
        let PlatformOutput {
            cursor_icon,
            open_url,
            copied_text,
            events: _,                    // handled above
            mutable_text_under_cursor: _, // only used in eframe web
            ..
        } = platform_output;

        // screen reader
        // egui_ctx.options(|&options| {
        //     if options.screen_reader {
        //         // speak
        //     }
        // });

        // set ime position
        // let Pos2 { x, y } = self.cursor_pos;
        // window.set_ime_position(winit::dpi::LogicalPosition { x, y });

        self.pixels_per_point = egui_ctx.pixels_per_point();

        self.set_cursor_icon(self.cursor_in_window, window, cursor_icon);

        #[cfg(feature = "clipboard")]
        {
            if !copied_text.is_empty() {
                self.copy_to_clipboard(&copied_text);
            }
        }

        #[cfg(feature = "webbrowser")]
        {
            if let Some(open_url) = open_url {
                open_url_in_browser(&open_url.url);
            }
        }
    }

    pub fn handle_event(&mut self, window: &mut PWindow, events: &GlfwReceiver<(f64, WindowEvent)>) {
        use glfw::WindowEvent::*;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close => window.set_should_close(true),
                _ => {
                    match event {
                        Focus(is_focus) => {
                            self.focus = is_focus;
                        }
                        Iconify(is_minimized) => {
                            self.minimized = is_minimized;
                        }
                        FramebufferSize(width, height) => {
                            self.raw_input.screen_rect = Some(
                                Rect::from_min_size(Pos2::new(0f32, 0f32), vec2(width as f32, height as f32) / self.pixels_per_point,
                                ));
                        }

                        MouseButton(mouse_btn, action, _) => {
                            self.raw_input.events.push(egui::Event::PointerButton {
                                pos: self.cursor_pos,
                                button: match mouse_btn {
                                    glfw::MouseButtonLeft => egui::PointerButton::Primary,
                                    glfw::MouseButtonRight => egui::PointerButton::Secondary,
                                    glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                                    _ => unreachable!(),
                                },
                                pressed: action == glfw::Action::Press,
                                modifiers: self.modifiers
                            });
                        }

                        CursorPos(x_offset, y_offset) => {
                            self.cursor_pos = pos2(x_offset as f32, y_offset as f32) / self.pixels_per_point;
                            self.raw_input
                                .events
                                .push(egui::Event::PointerMoved(self.cursor_pos));
                        }

                        CursorEnter(is_entered) => {
                            self.cursor_in_window = is_entered;
                        }

                        Scroll(x_offset, y_offset) => {
                            let points_per_scroll_line = 50.0; // Scroll speed decided by consensus: https://github.com/emilk/egui/issues/461
                            let mut delta = vec2(x_offset as f32, y_offset as f32) * points_per_scroll_line;
                            delta.x *= -1.0; // Winit has inverted hscroll. Remove this line when we update winit after https://github.com/rust-windowing/winit/pull/2105 is merged and released

                            if self.modifiers.ctrl || self.modifiers.command {
                                // Treat as zoom instead:
                                let factor = (delta.y / 200.0).exp();
                                self.raw_input.events.push(egui::Event::Zoom(factor));
                            } else if self.modifiers.shift {
                                // Treat as horizontal scrolling.
                                // Note: one Mac we already get horizontal scroll events when shift is down.
                                self.raw_input
                                    .events
                                    .push(egui::Event::Scroll(egui::vec2(delta.x + delta.y, 0.0)));
                            } else {
                                self.raw_input.events.push(egui::Event::Scroll(delta));
                            }
                        }

                        Key(keycode, _scancode, action, keymod) => {
                            self.modifiers = translate_modifiers(keymod);
                            let pressed = action == glfw::Action::Press;
                            let repeat = action == glfw::Action::Repeat;
                            if pressed {
                                if is_cut_command(self.modifiers, keycode) {
                                    self.raw_input.events.push(Event::Cut);
                                } else if is_copy_command(self.modifiers, keycode) {
                                    self.raw_input.events.push(Event::Copy);
                                } else if is_paste_command(self.modifiers, keycode) {
                                    #[cfg(feature = "clipboard")]
                                    {
                                        if let Some(content) = self.get_clipboard_content() {
                                            self.raw_input.events.push(Event::Paste(content));
                                        }
                                    }
                                }
                            }

                            if let Some(key) = translate_virtual_key_code(keycode) {
                                self.raw_input.events.push(Event::Key {
                                    key,
                                    physical_key: None,
                                    pressed,
                                    repeat,
                                    modifiers: self.modifiers,
                                });
                            }
                        }

                        Char(c) => {
                            self.raw_input.events.push(Event::Text(c.to_string()));
                        }

                        Maximize(is_maximized) => {
                            self.maximized = is_maximized;
                        }

                        // Pos(i32, i32),
                        // Size(i32, i32),
                        // Close,
                        // Refresh,
                        // CharModifiers(char, Modifiers),
                        // FileDrop(Vec<PathBuf>),
                        // Maximize(bool),
                        // ContentScale(f32, f32),
                        _ => {}
                    }
                }
            }
        }
    }
    
    pub fn set_cursor_icon(&mut self, in_window: bool, window: &mut PWindow, cursor_icon: CursorIcon) {
        self.cursor_current_icon = cursor_icon;
        if cursor_icon == CursorIcon::Default || cursor_icon == CursorIcon::None {
            return;
        }
        if let Some(cursor) = translate_cursor(cursor_icon) {
            window.set_cursor_mode(glfw::CursorMode::Normal);
            unsafe {
                if in_window {
                    let cursor = winuser::LoadCursorW(ptr::null_mut(), cursor.to_windows_cursor());
                    winuser::SetCursor(cursor);
                }
            }
        } else {
            window.set_cursor_mode(glfw::CursorMode::Hidden);
        }
    }
    
    pub fn get_clipboard_content(&mut self) -> Option<String> {
        if let Some(clipboard) = self.clipboard.as_mut() {
            if let Ok(content) = clipboard.get_contents() {
                if !content.is_empty() {
                    return Some(content.as_str().replace("\r\n", "\n"));
                }
            }
        }
        None
    }

    pub fn copy_to_clipboard(&mut self, copy_text: &str) {
        if let Some(clipboard) = self.clipboard.as_mut() {
            let result = clipboard.set_contents(copy_text.to_string());
            if result.is_err() {
                dbg!("Unable to set clipboard content.");
            }
        }
    }
}

fn open_url_in_browser(_url: &str) {
    if let Err(err) = webbrowser::open(_url) {
        dbg!("Failed to open url: {}", err);
    }
}

