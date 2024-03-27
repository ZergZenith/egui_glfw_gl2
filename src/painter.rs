use egui::{emath::Rect, epaint::{Mesh, Primitive}, Color32, TextureFilter, TextureId};

use std::ffi::{c_uint, c_void, CString};
use gl33::*;
use gl33::global_loader::*;
use crate::egui_shader::{FRAGMENT, VERTEX};

fn compile_shader(src: &str, ty: GLenum) -> c_uint {
    let shader = unsafe { glCreateShader(ty) };

    let c_str = CString::new(src.as_bytes()).unwrap();
    unsafe {
        glShaderSource(shader, 1, &c_str.as_ptr().cast(), core::ptr::null());
        glCompileShader(shader);
    }

    let mut status = 0;
    unsafe {
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut status);
    }

    if status != GL_TRUE.0 as _ {
        let mut len = 0;
        unsafe {
            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut len);
        }

        let mut buf = vec![0; len as usize];

        unsafe {
            glGetShaderInfoLog(shader, len, core::ptr::null_mut(), buf.as_mut_ptr().cast(), );
        }

        panic!(
            "{}",
            core::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
        );
    }

    shader
}

fn link_program(vs: c_uint, fs: c_uint) -> c_uint {
    let program = unsafe { glCreateProgram() };

    unsafe {
        glAttachShader(program, vs);
        glAttachShader(program, fs);
        glLinkProgram(program);
    }

    let mut status = 0;
    unsafe {
        glGetProgramiv(program, GL_LINK_STATUS, &mut status);
    }

    if status != GL_TRUE.0 as _ {
        let mut len = 0;
        unsafe {
            glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut len);
        }

        let mut buf = vec![0; len as usize];

        unsafe {
            glGetProgramInfoLog(
                program,
                len,
                core::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
            );
        }

        panic!(
            "{}",
            core::str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
        );
    }

    program
}

pub struct UserTexture {
    size: (usize, usize),

    /// Pending upload (will be emptied later).
    pixels: Vec<u8>,

    /// Lazily uploaded
    gl_texture_id: Option<c_uint>,

    /// For user textures there is a choice between
    /// Linear (default) and Nearest.
    filtering: TextureFilter,

    /// User textures can be modified and this flag
    /// is used to indicate if pixel data for the
    /// texture has been updated.
    dirty: bool,
}

impl UserTexture {
    pub fn update_texture_part(
        &mut self,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        bytes: &[u8],
    ) {
        assert!(x_offset + width <= self.size.0 as _);
        assert!(y_offset + height <= self.size.1 as _);

        unsafe {
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_A, GL_RED.0 as _);

            glTexSubImage2D(
                GL_TEXTURE_2D,
                0,
                x_offset as _,
                y_offset as _,
                width as _,
                height as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                bytes.as_ptr() as *const _,
            );
        }

        self.dirty = true;
    }

    pub fn from_raw(id: u32) -> Self {
        Self {
            size: (0, 0),
            gl_texture_id: Some(id),
            filtering: TextureFilter::Linear,
            dirty: false,
            pixels: Vec::with_capacity(0),
        }
    }

    pub fn delete(&self) {
        if let Some(id) = &self.gl_texture_id {
            unsafe {
                glDeleteTextures(1, id as *const _);
            }
        }
    }
}

pub struct Painter {
    program: c_uint,

    vertex_array: c_uint,
    index_buffer: c_uint,
    pos_buffer: c_uint,
    tc_buffer: c_uint,
    color_buffer: c_uint,

    canvas_width: u32,
    canvas_height: u32,

    textures: std::collections::HashMap<TextureId, UserTexture>,
}

impl Painter {
    pub fn set_size(&mut self, w: u32, h: u32) {
        (self.canvas_width, self.canvas_height) = (w, h);
    }

    pub fn new(window: &mut glfw::Window) -> Painter {
        let vs = compile_shader(VERTEX, GL_VERTEX_SHADER);
        let fs = compile_shader(FRAGMENT, GL_FRAGMENT_SHADER);

        let program = link_program(vs, fs);

        let mut vertex_array = 0;
        let mut index_buffer = 0;
        let mut pos_buffer = 0;
        let mut tc_buffer = 0;
        let mut color_buffer = 0;
        unsafe {
            glGenVertexArrays(1, &mut vertex_array);
            glBindVertexArray(vertex_array);
            glGenBuffers(1, &mut index_buffer);
            glGenBuffers(1, &mut pos_buffer);
            glGenBuffers(1, &mut tc_buffer);
            glGenBuffers(1, &mut color_buffer);
        }

        let (canvas_width, canvas_height) = window.get_size();

        Painter {
            program,

            vertex_array,
            index_buffer,
            pos_buffer,
            tc_buffer,
            color_buffer,

            canvas_width: canvas_width as _,
            canvas_height: canvas_height as _,

            textures: Default::default(),
        }
    }

    pub fn paint_and_update_textures(
        &mut self,
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.set_texture(*id, image_delta);
        }

        self.paint_primitives(pixels_per_point, clipped_primitives);

        for &id in &textures_delta.free {
            self.free_texture(id);
        }
    }

    /// Main entry-point for painting a frame.
    pub fn paint_primitives(
        &mut self,
        pixels_per_point: f32,
        clipped_primitives: &[egui::ClippedPrimitive],
    ) {
        self.upload_user_textures();

        unsafe {
            //Let OpenGL know we are dealing with SRGB colors so that it
            //can do the blending correctly. Not setting the framebuffer
            //leads to darkened, oversaturated colors.
            glEnable(GL_FRAMEBUFFER_SRGB);

            glEnable(GL_SCISSOR_TEST);
            glEnable(GL_BLEND);
            glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA); // premultiplied alpha
            glUseProgram(self.program);
            glActiveTexture(GL_TEXTURE0);
        }

        let u_screen_size = CString::new("u_screen_size").unwrap();
        let u_screen_size_loc = unsafe { glGetUniformLocation(self.program, u_screen_size.as_ptr().cast()) };
        let screen_size_points = egui::vec2(self.canvas_width as f32, self.canvas_height as f32) / pixels_per_point;

        unsafe {
            glUniform2f(
                u_screen_size_loc,
                screen_size_points.x,
                screen_size_points.y,
            );
        }

        let u_sampler = CString::new("u_sampler").unwrap();
        let u_sampler_loc = unsafe { glGetUniformLocation(self.program, u_sampler.as_ptr().cast()) };
        unsafe {
            glUniform1i(u_sampler_loc, 0);
            glViewport(0, 0, self.canvas_width as i32, self.canvas_height as i32);
        }

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in clipped_primitives
        {
            match primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(mesh, clip_rect, pixels_per_point);
                    unsafe {
                        glDisable(GL_SCISSOR_TEST);
                    }
                }

                Primitive::Callback(_) => {
                    panic!("Custom rendering callbacks are not implemented in egui_glium");
                }
            }
        }

        unsafe {
            glDisable(GL_FRAMEBUFFER_SRGB);
        }
    }

    pub fn new_opengl_texture(&mut self, openl_id: u32) -> egui::TextureId {
        let id = egui::TextureId::User(self.textures.len() as u64);

        self.textures.insert(id, UserTexture::from_raw(openl_id));

        id
    }

    pub fn new_user_texture(
        &mut self,
        size: (usize, usize),
        srgba_pixels: &[Color32],
        filtering: TextureFilter,
    ) -> egui::TextureId {
        assert_eq!(size.0 * size.1, srgba_pixels.len());

        let pixels: Vec<u8> = srgba_pixels.iter().flat_map(|a| a.to_array()).collect();
        let id = egui::TextureId::User(self.textures.len() as u64);

        self.textures.insert(
            id,
            UserTexture {
                size,
                pixels,
                gl_texture_id: None,
                filtering,
                dirty: true,
            },
        );

        id
    }

    pub fn update_user_texture_data(&mut self, texture_id: &egui::TextureId, pixels: &[Color32]) {
        let texture = self
            .textures
            .get_mut(texture_id)
            .expect("Texture with id has not been created");

        texture.pixels = pixels.iter().flat_map(|a| a.to_array()).collect();
        texture.dirty = true;
    }

    fn paint_mesh(&self, mesh: &Mesh, clip_rect: &Rect, pixels_per_point: f32) {
        debug_assert!(mesh.is_valid());

        if let Some(it) = self.textures.get(&mesh.texture_id) {
            unsafe {
                glBindTexture(
                    GL_TEXTURE_2D,
                    it.gl_texture_id
                        .expect("Texture should have a valid OpenGL id now"),
                );
            }

            let screen_size_pixels =
                egui::vec2(self.canvas_width as f32, self.canvas_height as f32);

            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;
            let clip_min_x = clip_min_x.clamp(0.0, screen_size_pixels.x);
            let clip_min_y = clip_min_y.clamp(0.0, screen_size_pixels.y);
            let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size_pixels.x);
            let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size_pixels.y);
            let clip_min_x = clip_min_x.round() as i32;
            let clip_min_y = clip_min_y.round() as i32;
            let clip_max_x = clip_max_x.round() as i32;
            let clip_max_y = clip_max_y.round() as i32;

            //scissor Y coordinate is from the bottom
            unsafe {
                glScissor(
                    clip_min_x,
                    self.canvas_height as i32 - clip_max_y,
                    clip_max_x - clip_min_x,
                    clip_max_y - clip_min_y,
                );
            }

            let indices: Vec<u16> = mesh.indices.iter().map(move |idx| *idx as u16).collect();
            let indices_len = indices.len();
            let vertices_len = mesh.vertices.len();

            unsafe {
                glBindVertexArray(self.vertex_array);
                glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, self.index_buffer);
                glBufferData(
                    GL_ELEMENT_ARRAY_BUFFER,
                    (indices_len * core::mem::size_of::<u16>()) as isize,
                    //mem::transmute(&indices.as_ptr()),
                    indices.as_ptr().cast(),
                    GL_STREAM_DRAW,
                );
            }

            let mut positions: Vec<f32> = Vec::with_capacity(2 * vertices_len);
            let mut tex_coords: Vec<f32> = Vec::with_capacity(2 * vertices_len);
            let mut colors: Vec<u8> = Vec::with_capacity(4 * vertices_len);
            for v in &mesh.vertices {
                positions.push(v.pos.x);
                positions.push(v.pos.y);

                tex_coords.push(v.uv.x);
                tex_coords.push(v.uv.y);

                colors.push(v.color[0]);
                colors.push(v.color[1]);
                colors.push(v.color[2]);
                colors.push(v.color[3]);
            }

            unsafe {
                glBindBuffer(GL_ARRAY_BUFFER, self.pos_buffer);
                glBufferData(
                    GL_ARRAY_BUFFER,
                    (positions.len() * core::mem::size_of::<f32>()) as isize,
                    //mem::transmute(&positions.as_ptr()),
                    positions.as_ptr().cast(),
                    GL_STREAM_DRAW,
                );
            }

            let a_pos = CString::new("a_pos").unwrap();
            let a_pos_loc = unsafe { glGetAttribLocation(self.program, a_pos.as_ptr().cast()) };
            assert!(a_pos_loc >= 0);
            let a_pos_loc = a_pos_loc as u32;

            let stride = 0;
            unsafe {
                glVertexAttribPointer(
                    a_pos_loc,
                    2,
                    GL_FLOAT,
                    GL_FALSE.0 as _,
                    stride,
                    core::ptr::null(),
                );
                glEnableVertexAttribArray(a_pos_loc);

                glBindBuffer(GL_ARRAY_BUFFER, self.tc_buffer);
                glBufferData(
                    GL_ARRAY_BUFFER,
                    (tex_coords.len() * core::mem::size_of::<f32>()) as isize,
                    //mem::transmute(&tex_coords.as_ptr()),
                    tex_coords.as_ptr().cast(),
                    GL_STREAM_DRAW,
                );
            }

            let a_tc = CString::new("a_tc").unwrap();
            let a_tc_loc = unsafe { glGetAttribLocation(self.program, a_tc.as_ptr().cast()) };
            assert!(a_tc_loc >= 0);
            let a_tc_loc = a_tc_loc as u32;

            let stride = 0;
            unsafe {
                glVertexAttribPointer(
                    a_tc_loc,
                    2,
                    GL_FLOAT,
                    GL_FALSE.0 as _,
                    stride,
                    core::ptr::null(),
                );
                glEnableVertexAttribArray(a_tc_loc);

                glBindBuffer(GL_ARRAY_BUFFER, self.color_buffer);
                glBufferData(
                    GL_ARRAY_BUFFER,
                    (colors.len() * core::mem::size_of::<u8>()) as isize,
                    //mem::transmute(&colors.as_ptr()),
                    colors.as_ptr().cast(),
                    GL_STREAM_DRAW,
                );
            }

            let a_srgba = CString::new("a_srgba").unwrap();
            let a_srgba_loc = unsafe { glGetAttribLocation(self.program, a_srgba.as_ptr().cast()) };
            assert!(a_srgba_loc >= 0);
            let a_srgba_loc = a_srgba_loc as u32;

            let stride = 0;
            unsafe {
                glVertexAttribPointer(
                    a_srgba_loc,
                    4,
                    GL_UNSIGNED_BYTE,
                    GL_FALSE.0 as _,
                    stride,
                    core::ptr::null(),
                );
                glEnableVertexAttribArray(a_srgba_loc);

                glDrawElements(GL_TRIANGLES, indices_len as i32, GL_UNSIGNED_SHORT, core::ptr::null(), );
                glDisableVertexAttribArray(a_pos_loc);
                glDisableVertexAttribArray(a_tc_loc);
                glDisableVertexAttribArray(a_srgba_loc);
            }
        }
    }

    pub fn set_texture(&mut self, tex_id: egui::TextureId, delta: &egui::epaint::ImageDelta) {
        let [w, h] = delta.image.size();

        if let Some([x, y]) = delta.pos {
            if let Some(texture) = self.textures.get_mut(&tex_id) {
                match &delta.image {
                    egui::ImageData::Color(image) => {
                        assert_eq!(
                            image.width() * image.height(),
                            image.pixels.len(),
                            "Mismatch between texture size and texel count"
                        );

                        let data: Vec<u8> =
                            image.pixels.iter().flat_map(|a| a.to_array()).collect();

                        texture.update_texture_part(x as _, y as _, w as _, h as _, &data);
                    }

                    egui::ImageData::Font(image) => {
                        assert_eq!(
                            image.width() * image.height(),
                            image.pixels.len(),
                            "Mismatch between texture size and texel count"
                        );

                        let gamma = Some(1.0f32);
                        let data: Vec<u8> = image
                            .srgba_pixels(gamma)
                            .flat_map(|a| a.to_array())
                            .collect();

                        texture.update_texture_part(x as _, y as _, w as _, h as _, &data);
                    }
                }
            } else {
                eprintln!("Failed to find egui texture {:?}", tex_id);
            }
        } else {
            let texture = match &delta.image {
                egui::ImageData::Color(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );

                    let pixels = image.pixels.iter().flat_map(|a| a.to_array()).collect();

                    UserTexture {
                        size: (w, h),
                        pixels,
                        gl_texture_id: None,
                        filtering: TextureFilter::Linear,
                        dirty: true,
                    }
                }
                egui::ImageData::Font(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );

                    let gamma = Some(1.0f32);
                    let pixels = image
                        .srgba_pixels(gamma)
                        .flat_map(|a| a.to_array())
                        .collect();

                    UserTexture {
                        size: (w, h),
                        pixels,
                        gl_texture_id: None,
                        filtering: TextureFilter::Linear,
                        dirty: true,
                    }
                }
            };

            let previous = self.textures.insert(tex_id, texture);
            if let Some(previous) = previous {
                previous.delete();
            }
        }
    }

    fn upload_user_textures(&mut self) {
        self.textures
            .values_mut()
            .filter(|user_texture| user_texture.gl_texture_id.is_none() || user_texture.dirty)
            .for_each(|user_texture| {
                let pixels = std::mem::take(&mut user_texture.pixels);

                match user_texture.gl_texture_id {
                    Some(texture) => unsafe {
                        glBindTexture(GL_TEXTURE_2D, texture);
                    },

                    None => {
                        let mut gl_texture = 0;
                        unsafe {
                            glGenTextures(1, &mut gl_texture);
                            glBindTexture(GL_TEXTURE_2D, gl_texture);
                            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE.0 as _, );
                            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE.0 as _, );
                        }

                        match user_texture.filtering {
                            TextureFilter::Nearest => unsafe {
                                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as _, );
                                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as _, );
                            },

                            TextureFilter::Linear => unsafe {
                                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR.0 as _, );
                                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as _, );
                            },
                        }
                        user_texture.gl_texture_id = Some(gl_texture);
                    }
                }

                if !pixels.is_empty() {
                    unsafe {
                        glTexImage2D(
                            GL_TEXTURE_2D,
                            0,
                            GL_RGBA.0 as _,
                            user_texture.size.0 as i32,
                            user_texture.size.1 as i32,
                            0,
                            GL_RGBA,
                            GL_UNSIGNED_BYTE,
                            pixels.as_ptr() as *const c_void,
                        );
                    }
                }

                user_texture.dirty = false;
            });
    }

    pub fn free_texture(&mut self, tex_id: egui::TextureId) {
        if let Some(old_tex) = self.textures.remove(&tex_id) {
            old_tex.delete();
        }
    }
}
