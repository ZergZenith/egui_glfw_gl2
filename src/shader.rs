use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::os::raw::{c_float, c_int, c_uint};
use std::path::Path;
use std::rc::Rc;

use cgmath::{Matrix, Matrix4};
use cgmath::num_traits::ToPrimitive;
use gl33::*;
use gl33::global_loader::*;
use regex::Regex;

pub struct ShaderSet {
    shaders: Vec<Rc<Shader>>
}

impl ShaderSet {
    pub fn new(file_list: Vec<&str>) -> Self {
        let mut shaders : Vec<Rc<Shader>> = Vec::new();
        for path_str in &file_list {
            shaders.push(Rc::new(Shader::new(path_str)));
        }
        ShaderSet {
            shaders
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rc<Shader>> {
        self.shaders.get(index)
    }
}

#[derive(Clone, Debug)]
pub struct Shader {
    shader_program_id: c_uint,

    file_path: String,
    vertex_src: String,
    fragment_src: String,
}

impl Shader {
    pub(crate) fn new(file_path: &str) -> Self {
        let (vertex_src, fragment_src) = load_shader(file_path);
        let mut shader = Self {
            shader_program_id: 0,
            file_path: file_path.to_string(),
            vertex_src,
            fragment_src
        };
        shader.compile();
        shader
    }

    fn compile(&mut self) {
        unsafe {
            // load and compile the vertex shader
            let vertex_id = glCreateShader(GL_VERTEX_SHADER);
            assert_ne!(vertex_id, 0);
            // vertex_id the shader source to the GPU
            glShaderSource(
                vertex_id,                  // shader id
                1,                           // number of shaders
                &self.vertex_src.as_bytes().as_ptr().cast(),    // the shader source
                &(self.vertex_src.len().try_into().unwrap())    // the length of the source
            );
            glCompileShader(vertex_id);
            self.check_shader_result(vertex_id, GL_COMPILE_STATUS, "Vertex");

            let fragment_id = glCreateShader(GL_FRAGMENT_SHADER);
            glShaderSource(
                fragment_id,                // shader id
                1,                           // number of shaders
                &self.fragment_src.as_bytes().as_ptr().cast(),  // the shader source
                &(self.fragment_src.len().try_into().unwrap())  // the length of the source
            );
            assert_ne!(fragment_id, 0);
            glCompileShader(fragment_id);
            self.check_shader_result(fragment_id, GL_COMPILE_STATUS, "Fragment");

            // Create an empty program
            self.shader_program_id = glCreateProgram();
            assert_ne!(self.shader_program_id, 0);
            // Attach the vertex and fragment shaders to the program
            glAttachShader(self.shader_program_id, vertex_id);
            glAttachShader(self.shader_program_id, fragment_id);
            // Link the program
            glLinkProgram(self.shader_program_id);
            self.check_shader_result(self.shader_program_id, GL_LINK_STATUS, "Program Link");

            glDeleteShader(vertex_id);
            glDeleteShader(fragment_id);
        }
    }

    unsafe fn check_shader_result(&self, id: c_uint, pname: GLenum, name: &str) {
        let mut success = 0;
        if pname == GL_LINK_STATUS {
            glGetProgramiv(id, GL_LINK_STATUS, &mut success);
        } else {
            glGetShaderiv(id, GL_COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            if pname == GL_LINK_STATUS {
                glGetProgramInfoLog(
                    id,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
            } else {
                glGetShaderInfoLog(
                    id,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
            }
            v.set_len(log_len.try_into().unwrap());
            panic!("{} Compile Error: {}", name, String::from_utf8_lossy(&v));
        }
    }

    pub fn attach(&self) {
        glUseProgram(self.shader_program_id);
    }

    pub fn detach(&self) {
        glUseProgram(0);
    }

    pub fn get_uniform_location(&self, name: &str) -> c_int {
        unsafe {
            let cstr = CString::new(name).unwrap();
            glGetUniformLocation(self.shader_program_id, cstr.as_ptr().cast())
        }
    }

    pub fn upload_mat4f(&self, name: &str, mat: Matrix4<f32>) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let location = glGetUniformLocation(self.shader_program_id, cstr.as_ptr().cast());
            assert_ne!(location, -1, "Error: error while upload_mat4f for shader");
            glUniformMatrix4fv(location, 1, false as u8, mat.as_ptr());
        }
    }

    pub fn upload_int_array(&self, name: &str, values: Vec<u32>) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let location = glGetUniformLocation(self.shader_program_id, cstr.as_ptr().cast());
            let len = values.len().to_isize().expect("Error: error while cast usize to isize");
            glUniform1iv(location, len as c_int, values.as_slice().as_ptr().cast());
        }
    }

    pub fn upload_int(&self, location: c_int, value: usize) {
        unsafe {
            glUniform1i(location, value as c_int);
        }
    }

    pub fn upload_float(&self, location: c_int, value: f32) {
        unsafe {
            glUniform1f(location, value as c_float);
        }
    }
}

impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.file_path.eq(&other.file_path)
    }

    fn ne(&self, other: &Self) -> bool {
        self.file_path.ne(&other.file_path)
    }
}

fn load_shader(file_path: &str) -> (String, String) {
    let path = Path::new(file_path);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Error: couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut source = String::new();
    if let Err(why) = file.read_to_string(&mut source) {
        panic!("Error: Couldn't read shader file {}: {}", display, why);
    }

    let split_string = Regex::new(r"(#type)( )+([a-zA-Z]+)")
        .unwrap()
        .split(&source.to_owned())
        .filter_map(|x| match x {
            "" | "\r\n" => None,
            _ => Some(x.to_string())
        })
        .collect::<Vec<String>>();
    if split_string.len() != 2 {
        panic!("Error: shader file format error");
    }

    let index = source.find("#type").unwrap() + 6;
    let eol = source[index..].find("\r\n").unwrap() + index + 2;
    let first_pattern = source[index..eol].trim();

    let index = source[eol..].find("#type").unwrap() + eol + 6;
    let eol =  source[index..].find("\r\n").unwrap() + index + 2;
    let second_pattern = source[index..eol].trim();

    let (mut vertex_src, mut fragment_src): (Option<String>, Option<String>) = (None, None);
    match first_pattern {
        "vertex" => vertex_src = Some(split_string.get(0).unwrap().trim().to_string()),
        "fragment" => fragment_src = Some(split_string.get(0).unwrap().trim().to_string()),
        other=> panic!("Error: Unexpected token '{}'", other)
    };

    match second_pattern {
        "vertex" => vertex_src = Some(split_string.get(1).unwrap().trim().to_string()),
        "fragment" => fragment_src = Some(split_string.get(1).unwrap().trim().to_string()),
        other=> panic!("Error: Unexpected token '{}'", other)
    };

    assert_ne!(vertex_src, None, "Error: Vertex shader source not found!");
    assert_ne!(fragment_src, None, "Error: Fragment shader source not found!");

    (vertex_src.unwrap(), fragment_src.unwrap())
}
