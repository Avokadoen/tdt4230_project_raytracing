
use cgmath::Vector3;

use super::{InitializeErr, program::Program, vao::VertexArrayObject};

pub const EMPTY: i32 = 0;
pub const PARENT: i32 = 1;
pub const LEAF: i32 = 2;

pub struct Octree {
    min_point: Vector3<f32>,
    scale: f32,
    current_max_depth: i32,
    cell_count: i32,
    pub vao: VertexArrayObject
}

impl Octree {
    // TODO: cell_count can be computed by buffer len
    pub fn new(min_point: Vector3<f32>, scale: f32, current_max_depth: i32, cell_count: i32, vao: VertexArrayObject) -> Result<Octree, InitializeErr> {
        Ok(Octree {
            min_point,
            scale, 
            current_max_depth,
            cell_count,
            vao
        })
    }

    pub fn update_pos_scale(&self, rt_program: &mut Program) -> Result<(), InitializeErr> {
        if let Err(e) = rt_program.set_vector3_f32("octree.min_point", self.min_point) {
            return Err(e);
        }

        if let Err(e) = rt_program.set_vector3_i32("octree.indirect_pool_size", Vector3::new(self.cell_count * 2, 2, 2)) {
            return Err(e);
        }
        
        if let Err(e) = rt_program.set_f32("octree.scale", self.scale) {
            return Err(e);
        }

        if let Err(e) = rt_program.set_f32("octree.inv_scale", 1.0 / self.scale) {
            return Err(e);
        }

        if let Err(e) = rt_program.set_i32("octree.current_max_depth", self.current_max_depth) {
            return Err(e);
        }

        if let Err(e) = rt_program.set_i32("octree.cell_count", self.cell_count) {
            return Err(e);
        }

        // TODO: configurable
        if let Err(e) = rt_program.set_i32("octree.max_traversal_iter", 30) {
            return Err(e);
        }

        Ok(())
    }

    // pub fn generate_terrain()
}