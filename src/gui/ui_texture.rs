use std::cell::Cell;
use std::ffi::c_uint;

use egui::{TextureFilter, TextureOptions, TextureWrapMode};
use gl33::*;
use gl33::global_loader::*;

pub struct GuiTexture {
    texture_id: Cell<c_uint>,

    options: TextureOptions,

    size: [usize;2],
    // always be [] if not custom texture
    pixels: Vec<u8>,
    // always be false if not custom texture
    dirty: Cell<bool>
}

impl GuiTexture {
    pub fn new(texture_id: c_uint, options: TextureOptions, size: [usize;2], pixels: Vec<u8>, dirty: bool) -> Self {
        Self {
            texture_id: Cell::new(texture_id),
            options,
            size,
            pixels,
            dirty: Cell::new(dirty)
        }
    }

    pub fn texture_id(&self) -> c_uint {
        self.texture_id.get()
    }
    pub fn size(&self) -> [usize; 2] {
        self.size
    }
    pub fn dirty(&self) -> bool {
        self.dirty.get()
    }
    pub fn take_pixels(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pixels)
    }
    pub fn set_texture_id(&self, texture_id: c_uint) {
        self.texture_id.set(texture_id);
    }
    pub fn set_pixels(&mut self, pixels: Vec<u8>) {
        self.pixels = pixels;
    }
    pub fn set_dirty(&self, dirty: bool) {
        self.dirty.set(dirty);
    }
}

impl GuiTexture {
    pub fn gen_tex_and_bind(&self) {
        assert_eq!(self.texture_id(), 0);
        let mut texture_id = 0;
        unsafe {
            glGenTextures(1, &mut texture_id);
            assert_ne!(texture_id, 0);
            self.set_texture_id(texture_id);
            // bind
            glBindTexture(GL_TEXTURE_2D, texture_id);
            // settings
            let warap_mod = match &self.options.wrap_mode {
                TextureWrapMode::ClampToEdge => GL_CLAMP_TO_EDGE,
                TextureWrapMode::Repeat => GL_REPEAT,
                TextureWrapMode::MirroredRepeat => GL_MIRRORED_REPEAT
            };
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, warap_mod.0 as _);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, warap_mod.0 as _);
            glTexParameteri(GL_TEXTURE_2D,
                            GL_TEXTURE_MIN_FILTER,
                            match &self.options.minification {
                                TextureFilter::Nearest => GL_NEAREST,
                                TextureFilter::Linear => GL_LINEAR
                            }.0 as _
            );
            glTexParameteri(GL_TEXTURE_2D,
                            GL_TEXTURE_MAG_FILTER,
                            match &self.options.magnification {
                                TextureFilter::Nearest => GL_NEAREST,
                                TextureFilter::Linear => GL_LINEAR
                            }.0 as _
            );
        }
    }

    pub fn free(&self) {
        let texture = self.texture_id();
        if texture != 0 {
            unsafe {
                glDeleteTextures(1, texture as _);
            }
        }
    }
}

impl GuiTexture {
    pub fn upload(&self, data: Vec<u8>) {
        let texture_id = self.texture_id();
        assert_ne!(texture_id, 0);
        let [texture_width, texture_height] = self.size;
        unsafe {
            glBindTexture(GL_TEXTURE_2D, texture_id);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_SRGB8_ALPHA8.0 as _,
                texture_width as _,
                texture_height as _,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                data.as_ptr().cast()
            );
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }

    pub fn upload_sub(&self, x_offset: usize, y_offset: usize, texture_width: usize, texture_height: usize, data: Vec<u8>) {
        let texture_id = self.texture_id();
        assert_ne!(texture_id, 0);
        unsafe {
            glBindTexture(GL_TEXTURE_2D, texture_id);
            glTexSubImage2D(
                GL_TEXTURE_2D,
                0,
                x_offset as _,
                y_offset as _,
                texture_width as _,
                texture_height as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }
}
