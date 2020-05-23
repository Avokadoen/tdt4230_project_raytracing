use gl::types::{GLuint, GLenum};

#[allow(dead_code)]
pub struct Texture {
    id: u32,
    active: GLenum,
}

// TODO: impl Drop glDeleteTextures 

impl Texture {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[allow(dead_code)]
    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(self.active);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    #[allow(dead_code)]
    pub fn unbind(&self) {
        unsafe {
            gl::ActiveTexture(self.active);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    // TODO: program should have configureable texture size
    #[allow(dead_code)]
    pub fn new(active: GLenum, bind_slot: GLuint, internal_format: GLenum, gltype: GLenum) -> Self {
        let id = prep_texture(active);
        unsafe { 
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                internal_format as i32, 
                512,
                512,
                0, 
                gltype, 
                gl::UNSIGNED_BYTE, 
                std::ptr::null() 
            );
            gl::BindImageTexture(bind_slot, id, 0, gl::FALSE, 0, gl::READ_WRITE, internal_format);
        }

        Texture {
            id,
            active
        }
    }

    #[allow(dead_code)]
    pub fn from_image(image: image::RgbaImage, active: GLenum, bind_slot: GLuint, internal_format: GLenum, gltype: GLenum) -> Self {
        let flip_image = image::imageops::flip_vertical(&image);

        let id = prep_texture(active);
        
        unsafe { 
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                internal_format as i32, 
                512,
                512,
                0, 
                gltype, 
                gl::UNSIGNED_BYTE, 
                flip_image.into_raw().as_ptr() as *const std::ffi::c_void
            );
            gl::BindImageTexture(bind_slot, id, 0, gl::FALSE, 0, gl::READ_WRITE, internal_format);
        }

        Texture {
            id,
            active
        }
    }
}

fn prep_texture(active: GLenum) -> GLuint {
    let mut id: GLuint = 0;
    
    unsafe { 
        gl::ActiveTexture(active);
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    }

    id
}