
use std::collections::HashMap;
use std::ffi::c_uint;
use std::mem;
use std::ops::Deref;

use egui::{ClippedPrimitive, Color32, ImageData, Mesh, Rect, TextureFilter, TextureId, TextureOptions, TexturesDelta};
use egui::epaint::{ImageDelta, Primitive};
use gl33::*;
use gl33::global_loader::*;

use crate::gui::ui_texture::GuiTexture;
use crate::shader::Shader;

const POS_SIZE: i32 = 2;
const TEX_COORDS_SIZE: i32 = 2;
const COLOR_SIZE: i32 = 4;

const POS_OFFSET: i32 = 0;
const TEX_COORDS_OFFSET: i32 = POS_OFFSET + POS_SIZE * (mem::size_of::<f32>() as i32);
const COLOR_OFFSET: i32 = TEX_COORDS_OFFSET + TEX_COORDS_SIZE * (mem::size_of::<f32>() as i32);

const VERTEX_SIZE: i32 = POS_SIZE + TEX_COORDS_SIZE + COLOR_SIZE;
const VERTEX_SIZE_BYTES: i32 = (POS_SIZE + TEX_COORDS_SIZE) * (mem::size_of::<f32>() as i32) + COLOR_SIZE * (mem::size_of::<u8>() as i32);

struct Vertex {
    position: [f32; POS_SIZE as usize],
    coords: [f32; TEX_COORDS_SIZE as usize],
    color: [u8; COLOR_SIZE as usize]
}

pub struct GuiRender {
    shader: Shader,
    vao_id: c_uint,
    vbo_id: c_uint,
    ebo_id: c_uint,

    canvas_width: usize,
    canvas_height: usize,
    textures: HashMap<TextureId, GuiTexture>
}

impl GuiRender {
    pub fn new(width: usize, height: usize) -> Self {
        let shader = Shader::new("assets/shaders/egui.glsl");
        unsafe {
            let mut vao_id = 0;
            glGenVertexArrays(1, &mut vao_id);
            assert_ne!(vao_id, 0);
            glBindVertexArray(vao_id);

            let mut vbo_id = 0;
            glGenBuffers(1, &mut vbo_id);
            assert_ne!(vbo_id, 0);

            let mut ebo_id = 0;
            glGenBuffers(1, &mut ebo_id);
            assert_ne!(ebo_id, 0);

            GuiRender {
                shader,
                vao_id,
                vbo_id,
                ebo_id,

                canvas_width: width,
                canvas_height: height,

                textures: Default::default(),
            }
        }
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        (self.canvas_width, self.canvas_height) = (width, height);
    }
}

impl GuiRender {
    pub fn render(&mut self, pixels_per_point: f32, clipped_primitives: &[ClippedPrimitive], textures_delta: &TexturesDelta) {
        // 3. Get textures to be rendered from egui_ctx, bind and upload to GPU
        for (id, image_delta) in &textures_delta.set {
            self.upload_egui_texture(*id, image_delta);
        }
        // 4. Get Custom textures, bind, and upload to GPU
        self.upload_custom_texture();
        // 5. Get vertex and other related data to be rendered from egui_ctx, do render
        self.paint(pixels_per_point, clipped_primitives);
        // 6. Get materials that need to be released from egui_ctx and release them
        for id in &textures_delta.free {
            self.free_texture(id);
        }
    }
}

impl GuiRender {
    pub fn new_texture(&mut self, size: (usize, usize), srgba_pixels: &[Color32], options: TextureOptions) -> egui::TextureId {
        assert_eq!(size.0 * size.1, srgba_pixels.len());

        let pixels: Vec<u8> = srgba_pixels.iter().flat_map(|a| a.to_array()).collect();
        let id = TextureId::User(self.textures.len() as u64);
        self.textures.insert(
            id,
            GuiTexture::new(
                0,
                options,
                [size.0, size.1],
                pixels,
                true
            ),
        );
        id
    }

    pub fn update_texture(&mut self, texture_id: &TextureId, pixels: &[Color32]) {
        let texture = self
            .textures
            .get_mut(texture_id)
            .expect("Texture with id has not been created");

        texture.set_pixels(pixels.iter().flat_map(|a| a.to_array()).collect());
        texture.set_dirty(true);
    }
}

impl GuiRender {
    pub fn upload_egui_texture(&mut self, id: TextureId, delta: &ImageDelta) {
        let options = delta.options;
        let [texture_width, texture_height] = delta.image.size();
        let data: Vec<u8> = match &delta.image {
            ImageData::Color(image) => {
                assert_eq!(image.width() * image.height(), image.pixels.len(), "Mismatch between texture size and texel count");
                image.pixels
                    .iter()
                    .flat_map(|a| a.to_array())
                    .collect()
            }
            ImageData::Font(font) => font
                .srgba_pixels(Some(0.4f32))
                .flat_map(|a| a.to_array())
                .collect()
        };

        // sub texture
        if let Some(delta_pos) = delta.pos {
            let texture = self.textures.get_mut(&id).expect("Texture not found.");
            let [x_offset, y_offset] = delta_pos;
            let size = texture.size();
            assert!(x_offset + texture_width <= size[0] as _);
            assert!(y_offset + texture_height <= size[1] as _);
            // upload sub texture
            texture.upload_sub(x_offset, y_offset, texture_width, texture_height, data);
            return;
        }
        // one picture as whole texture
        let mut texture = GuiTexture::new(
            0,
            options,
            [texture_width, texture_height],
            vec![],
            false
        );
        texture.gen_tex_and_bind();
        texture.upload(data);
        if let Some(old_texture) = self.textures.insert(id, texture) {
            old_texture.free();
        }
    }

    pub fn upload_custom_texture(&mut self) {
        for (_, texture) in self.textures.iter_mut() {
            if !texture.dirty() {
                continue;
            }
            if texture.texture_id() == 0 {
                texture.gen_tex_and_bind();
            }
            let data = texture.take_pixels();
            texture.upload(data);
            texture.set_dirty(false);
        }
    }

    pub fn free_texture(&mut self, id: &TextureId) {
        if let Some(old_tex) = self.textures.remove(id) {
            old_tex.free();
        }
    }
}

impl GuiRender {
    fn paint(&self, pixels_per_point: f32, clipped_primitives: &[ClippedPrimitive]) {
        unsafe {
            glEnable(GL_SCISSOR_TEST);

            // bind shader
            self.shader.attach();
            glActiveTexture(GL_TEXTURE0);
            // upload uniform
            let screen_size_points = egui::vec2(self.canvas_width as f32, self.canvas_height as f32) / pixels_per_point;
            let u_screen_size_loc = self.shader.get_uniform_location("uScreenSize");
            glUniform2f(u_screen_size_loc, screen_size_points.x, screen_size_points.y);
            // upload uniform
            let u_sampler_loc = self.shader.get_uniform_location("uSampler");
            glUniform1i(u_sampler_loc, 0);
            glViewport(0, 0, self.canvas_width as i32, self.canvas_height as i32);
        }

        for ClippedPrimitive { clip_rect, primitive} in clipped_primitives {
            match primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(mesh, clip_rect, pixels_per_point);
                }
                Primitive::Callback(_) => {
                    panic!("Custom rendering callbacks are not implemented in egui_glium");
                }
            }
        }

        unsafe {
            glDisable(GL_SCISSOR_TEST);
        }
    }

    fn paint_mesh(&self, mesh: &Mesh, clip_rect: &Rect, pixels_per_point: f32) {
        debug_assert!(mesh.is_valid());

        if let Some(texture) = self.textures.get(&mesh.texture_id) {
            unsafe {
                glBindTexture(GL_TEXTURE_2D, texture.texture_id());
            }

            let screen_size_pixels = egui::vec2(self.canvas_width as f32, self.canvas_height as f32);
            // Transform clip rect to physical pixels:
            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;
            // Clamp:
            let clip_min_x = clip_min_x.clamp(0.0, screen_size_pixels.x);
            let clip_min_y = clip_min_y.clamp(0.0, screen_size_pixels.y);
            let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size_pixels.x);
            let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size_pixels.y);
            // Round to integer:
            let clip_min_x = clip_min_x.round() as i32;
            let clip_min_y = clip_min_y.round() as i32;
            let clip_max_x = clip_max_x.round() as i32;
            let clip_max_y = clip_max_y.round() as i32;

            //scissor Y coordinate is from the bottom
            unsafe {
                glScissor(clip_min_x, self.canvas_height as i32 - clip_max_y, clip_max_x - clip_min_x, clip_max_y - clip_min_y, );

                glBindVertexArray(self.vao_id);

                let mut vertices: Vec<Vertex> = Vec::with_capacity(mesh.vertices.len());
                for v in &mesh.vertices {
                    vertices.push(Vertex{
                        position: [v.pos.x, v.pos.y],
                        coords: [v.uv.x, v.uv.y],
                        color: v.color.to_array(),
                    });
                }

                glBindBuffer(GL_ARRAY_BUFFER, self.vbo_id);
                glBufferData(GL_ARRAY_BUFFER, mem::size_of_val(vertices.deref()) as isize, vertices.as_ptr().cast(), GL_STREAM_DRAW);

                let indices: Vec<u16> = mesh.indices.iter().map(move |idx| *idx as u16).collect();
                glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, self.ebo_id);
                glBufferData(GL_ELEMENT_ARRAY_BUFFER, mem::size_of_val(indices.deref()) as isize, indices.as_ptr().cast(), GL_STREAM_DRAW);

                glVertexAttribPointer(0, POS_SIZE, GL_FLOAT, GL_FALSE.0 as _, VERTEX_SIZE_BYTES, POS_OFFSET as *const _);
                glEnableVertexAttribArray(0);
                glVertexAttribPointer(1, TEX_COORDS_SIZE, GL_FLOAT, GL_FALSE.0 as _, VERTEX_SIZE_BYTES, TEX_COORDS_OFFSET as *const _);
                glEnableVertexAttribArray(1);
                glVertexAttribPointer(2, COLOR_SIZE, GL_UNSIGNED_BYTE, GL_FALSE.0 as _, VERTEX_SIZE_BYTES, COLOR_OFFSET as *const _);
                glEnableVertexAttribArray(2);

                glDrawElements(GL_TRIANGLES, indices.len() as _, GL_UNSIGNED_SHORT, core::ptr::null());

                glDisableVertexAttribArray(0);
                glDisableVertexAttribArray(1);
                glDisableVertexAttribArray(2);
            }
        }
    }
}