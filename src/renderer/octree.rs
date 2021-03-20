use gl::types::GLsizei;

use super::{InitializeErr, texture::Texture};

pub struct Octree {
    structure: Texture,
}

impl Octree {
    pub fn new(max_depth: u32) -> Result<Octree, InitializeErr> {
        // TODO: allocating worst case memory usage is probably a bad idea
        //       on bigger octrees i.e terrain.
        // max slots is always 2 * 2^n in a N^3-Tree 
        let dimention = 2 * 2i32.pow(max_depth);
        let structure = Texture::new_3d(
            gl::TEXTURE1,
            1,
            gl::RGBA16F,
            gl::RGB,
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