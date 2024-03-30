// Draws a simple white triangle
// based on the example from:
// https://github.com/brendanzab/gl-rs/blob/master/gl/examples/triangle.rs

use gl33::*;
use gl33::global_loader::*;
use std::{mem, ptr, str};

use std::ffi::{c_uint, CString};

#[allow(unconditional_panic)]
const fn illegal_null_in_string() {
    [][0]
}

#[doc(hidden)]
pub const fn validate_cstr_contents(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\0' {
            illegal_null_in_string();
        }
        i += 1;
    }
}

macro_rules! cstr {
    ( $s:literal ) => {{
        validate_cstr_contents($s.as_bytes());
        unsafe { std::mem::transmute::<_, &std::ffi::CStr>(concat!($s, "\0")) }
    }};
}

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
            glGetShaderInfoLog(shader, len, core::ptr::null_mut(), buf.as_mut_ptr().cast());
        }

        panic!("{}", core::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8"));
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
            glGetProgramInfoLog(program, len, core::ptr::null_mut(), buf.as_mut_ptr().cast());
        }

        panic!("{}", core::str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8"));
    }

    program
}

const VS_SRC: &str = "
#version 150
in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FS_SRC: &str = "
#version 150
out vec4 out_color;

void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}";

static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

pub struct Triangle {
    pub vs: c_uint,
    pub fs: c_uint,
    pub program: c_uint,
    pub vao: c_uint,
    pub vbo: c_uint,
}

impl Triangle {
    pub fn new() -> Self {
        let vs = compile_shader(VS_SRC, GL_VERTEX_SHADER);
        let fs = compile_shader(FS_SRC, GL_FRAGMENT_SHADER);
        let program = link_program(vs, fs);

        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            glGenVertexArrays(1, &mut vao);
            glGenBuffers(1, &mut vbo);
        }

        Triangle {
            vs,
            fs,
            program,
            vao,
            vbo,
        }
    }

    pub fn draw(&self) {
        unsafe {
            glBindVertexArray(self.vao);

            glBindBuffer(GL_ARRAY_BUFFER, self.vbo);
            glBufferData(GL_ARRAY_BUFFER, (VERTEX_DATA.len() * mem::size_of::<f32>()) as _, mem::transmute(&VERTEX_DATA[0]), GL_STATIC_DRAW, );
            glUseProgram(self.program);

            let c_out_color = cstr!("out_color");

            glBindFragDataLocation(self.program, 0, c_out_color.as_ptr().cast());

            let c_position = cstr!("position");
            let pos_attr = glGetAttribLocation(self.program, c_position.as_ptr().cast());

            glEnableVertexAttribArray(pos_attr as _);
            glVertexAttribPointer(pos_attr as _, 2, GL_FLOAT, GL_FALSE.0 as _, 0, ptr::null(), );

            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
    }
}

impl Drop for Triangle {
    fn drop(&mut self) {
        unsafe {
            glDeleteProgram(self.program);
            glDeleteShader(self.fs);
            glDeleteShader(self.vs);
            glDeleteBuffers(1, &self.vbo);
            glDeleteVertexArrays(1, &self.vao);
        }
    }
}
