// TODO: Document functions and structs with /// https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
// TODO: most of these modules are very much raw opengl, I should create a interface that is more trivial to be 
//       duplicated by other api's
pub mod camera;
pub mod shader;
pub mod program;
pub mod texture;
pub mod vao;
pub mod vbo;

mod utils;

pub enum Material {
    Lambertian = 0,
    Metal,
}