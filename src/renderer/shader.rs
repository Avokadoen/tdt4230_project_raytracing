
// Used for shaders
use std::ffi::{CStr};

// TODO: we should probably move shader and program into separate files/modules


use gl::types::{
    GLuint,
    GLenum,
    GLint,
    GLchar
};

// split shader into types i.e vertex, fragment and use some sort of polymorphic behaviour between them or 
// simply some generics
pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn from_source(
        source: &CStr,
        kind: GLenum
    ) -> Result<Shader, String> {
        let id = {
            let id = unsafe { gl::CreateShader(kind) };

            unsafe {
                gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
                gl::CompileShader(id);
            }
        
            let mut success: GLint = 1;
            unsafe {
                gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            }
        
            if success == 0 {
                let mut len: GLint = 0;
                unsafe {
                    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
                }
        
                let error = super::utils::create_whitespace_cstring_with_len(len as usize);
                unsafe {
                    gl::GetShaderInfoLog(
                        id,
                        len,
                        std::ptr::null_mut(),
                        error.as_ptr() as *mut GLchar
                    );
                }
        
                return Err(error.to_string_lossy().into_owned());
            }
            
            id
        };
        Ok(Shader { id })
    }

    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}



