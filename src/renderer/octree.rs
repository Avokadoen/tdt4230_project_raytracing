
use super::{InitializeErr, texture::Texture};

pub struct Octree {
    structure: Texture,
}

impl Octree {
    const MAX_DEPTH: u32 = 5;

    pub fn new(max_depth: u32) -> Result<Octree, InitializeErr> {
        if max_depth > 5 {
            let msg = String::from(format!("Max depth cant exceed {}", Octree::MAX_DEPTH));
            return Err(InitializeErr::InvalidArgument(msg));
        }

        // max slots is always 2 * 2^n in a N^3-Tree 
        let dimention = 2 * 2i32.pow(max_depth);
        let structure = Texture::new_3d(
            gl::TEXTURE1,
            1,
            gl::RGBA32F,
            gl::RGBA,
            dimention,
            dimention,
            dimention
        )?;

        Ok(Octree {
            structure
        })
    }

    pub fn bind(&self) {
        self.structure.bind();
    }

    pub fn unbind(&self) {
        self.structure.unbind();
    }

    // pub fn generate_terrain()
}