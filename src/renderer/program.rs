use std::collections::HashMap;

use gl::types::{
    GLuint,
    GLint,
    GLchar,
};

use super::{InitializeErr, check_for_gl_error, shader::Shader};
use crate::Resources;
// TODO: rename?
pub struct Program {
    id: GLuint,
    uniforms: HashMap<String, GLint> 
}

impl Program {
    pub fn id(&self) -> GLuint {
        return self.id;
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    // TODO: macro for this
    pub fn set_i32(&mut self, name: &str, value: i32) -> Result<(), InitializeErr> {
        match self.register_uniform(name) {
            Ok(()) => {
                unsafe {
                    gl::ProgramUniform1i(self.id, self.uniforms[name], value);
                }
                return Ok(())
            },
            Err(e) => Err(e.var_into_typed("i32"))
        }
    }

    pub fn set_vector3_f32(&mut self, name: &str, value: cgmath::Vector3<f32>) -> Result<(), InitializeErr> {
        match self.register_uniform(name) {
            Ok(()) => {
                unsafe {
                    gl::ProgramUniform3f(self.id, self.uniforms[name], value.x, value.y, value.z);
                }
                return Ok(());
            }
            Err(e) => Err(e.var_into_typed("vec3"))
        }
    }

    /// creates a program out of a folder path that contains both a fragment shader and vertex shader
    pub fn from_resources(res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_resources(res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(&shaders[..])
    }

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }

        unsafe { gl::LinkProgram(program_id); }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = super::utils::create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        // TODO: waste creating a hashmap every time just to have variables. Redesign this
        Ok(Program { id: program_id, uniforms: HashMap::new() })
    }
 
    // TODO: rename as it also checks if it exist
    fn register_uniform(&mut self, name: &str) -> Result<(), InitializeErr> {
        if !self.uniforms.contains_key(name) {
            use std::ffi::CString;
            let c_name = match CString::new(name) {
                Ok(s) => s,
                Err(e) => return Err(InitializeErr::InvalidCStr(e))
            };
            let uni_location = unsafe {
                gl::GetUniformLocation(self.id, c_name.as_ptr() as *const i8)
            };
            
            unsafe { check_for_gl_error()?; }
            
            if uni_location >= 0 {
                &self.uniforms.insert(String::from(name), uni_location);
            } else {
                return Err(InitializeErr::VariableNotFound(name.to_string()));
            }
        }

        return Ok(());
    }
}


impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}