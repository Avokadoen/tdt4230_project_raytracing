use core::fmt;

use gl::types::GLenum;

// TODO: Document functions and structs with /// https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
// TODO: most of these modules are very much raw opengl, I should create a interface that is more trivial to be 
//       duplicated by other api's
pub mod camera;
pub mod shader;
pub mod program;
pub mod texture;
pub mod vao;
pub mod vbo;
pub mod octree;
pub mod compute_shader;

mod utils;

pub enum Material {
    Lambertian = 0,
    Metal,
    Dielectric,
}

#[derive(Debug)]
pub enum InitializeErr {
    GL(GLenum),
    InvalidArgument(String),
}

impl fmt::Display for InitializeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InitializeErr::GL(code) => {
                match code {
                    &gl::INVALID_ENUM => write!(f, "gl error: invalid enum"),
                    &gl::INVALID_VALUE => write!(f, "gl error: invalid value"),
                    &gl::INVALID_OPERATION => write!(f, "gl error: invalid operation"),
                    _ => write!(f, "got gl error code: {}", code)
                }
            }
            InitializeErr::InvalidArgument(s) => write!(f, "{}", s),
        }
    }
}

pub unsafe fn check_for_gl_error() -> Result<(), InitializeErr> {
    let code = gl::GetError();
    if code != gl::NO_ERROR {
        return Err(InitializeErr::GL(code));
    }
    Ok(())
}