use gl::types::{GLenum, GLsizei, GLuint};

use super::{InitializeErr, check_for_gl_error};

pub struct Texture {
    id: u32,
    active: GLenum,
    width: i32,
    height: i32,
    depth: i32,
    target: GLenum,
}

// TODO: impl Drop glDeleteTextures 

impl Texture {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(self.active);
            gl::BindTexture(self.target, self.id);
            check_for_gl_error().unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn unbind(&self) {
        unsafe {
            gl::ActiveTexture(self.active);
            gl::BindTexture(self.target, 0);
        }
    }

    pub fn new_2d(active: GLenum, bind_slot: GLuint, internal_format: GLenum, format: GLenum, width: GLsizei, height: GLsizei) -> Result<Self, InitializeErr> {       
        let target = gl::TEXTURE_2D;
        let id = prep_texture(active, target)?;
        unsafe { 
            gl::TexImage2D(
                target, 
                0, 
                internal_format as i32, 
                width,
                height,
                0, 
                format, 
                gl::UNSIGNED_BYTE, 
                std::ptr::null() 
            );
            check_for_gl_error()?;
            gl::BindImageTexture(bind_slot, id, 0, gl::FALSE, 0, gl::READ_WRITE, internal_format);
            check_for_gl_error()?;
        }

        Ok(Texture {
            id,
            active,
            width,
            height,
            depth: 1,
            target,
        })
    }

    pub fn new_3d(active: GLenum, bind_slot: GLuint, internal_format: GLenum, format: GLenum, width: GLsizei, height: GLsizei, depth: GLsizei) -> Result<Self, InitializeErr> {       
        let target = gl::TEXTURE_3D;
        let id = prep_texture(active, target)?;
        unsafe { 
            gl::TexImage3D(
                target, 
                0, 
                internal_format as i32, 
                width,
                height,
                depth,
                0, 
                format, 
                gl::UNSIGNED_BYTE, 
                std::ptr::null() 
            );
            check_for_gl_error()?;
            gl::BindImageTexture(bind_slot, id, 0, gl::FALSE, 0, gl::READ_WRITE, internal_format);
            check_for_gl_error()?;
        }

        Ok(Texture {
            id,
            active,
            width,
            height,
            depth,
            target,
        })
    }
}

fn prep_texture(active: GLenum, target: GLenum) -> Result<GLuint, InitializeErr> {
    let mut id: GLuint = 0;
    
    unsafe { 
        gl::ActiveTexture(active);
        gl::GenTextures(1, &mut id);
        gl::BindTexture(target, id);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        check_for_gl_error()?;
    }

    Ok(id)
}