use core::fmt;
use std::{ffi::NulError, fmt::Debug};

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

// TODO: split this into different errors
#[derive(Debug)]
pub enum InitializeErr {
    GL(GLenum),
    VariableNotFound(String),
    TypedVariableNotFound(String, String),
    InvalidCStr(NulError),
}

impl InitializeErr {
    pub fn var_into_typed(self, type_str: &str) -> InitializeErr {
        match self {
            Self::VariableNotFound(name) => InitializeErr::TypedVariableNotFound(name, type_str.to_string()),
            e => e
        }
    }
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
            InitializeErr::VariableNotFound(name) => write!(f, "failed to locate uniform {}", name),
            InitializeErr::TypedVariableNotFound(name, utype) => write!(f, "failed to locate uniform {} with type {}", name, utype),
            InitializeErr::InvalidCStr(e) => write!(f, "{}", e),
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