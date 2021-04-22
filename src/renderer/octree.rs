
use cgmath::{Vector3, num_traits::Pow};

use super::{InitializeErr, compute_shader::ComputeShader, vao::VertexArrayObject};

pub const EMPTY: u32 = 0;
pub const PARENT: u32 = 1;
pub const LEAF: u32 = 2;

// TODO: builder?
pub struct Octree {
    min_point: Vector3<f32>,
    scale: f32,
    max_depth: i32,
    cell_count: i32,
    active_cell_count: i32,
    max_traversal_iter: i32,
    pub vao: VertexArrayObject,
    
    // distance between each block min point
    block_distance: f32, 
}

impl Octree {
    // TODO: cell_count can be computed by buffer len
    pub fn new(min_point: Vector3<f32>, scale: f32, max_depth: i32, cell_count: i32, active_cell_count: i32, max_traversal_iter: i32, vao: VertexArrayObject) -> Result<Octree, InitializeErr> {
        let block_distance = scale / 2f32.pow(max_depth);
        Ok(Octree {
            min_point,
            scale, 
            max_depth,
            cell_count,
            active_cell_count,
            max_traversal_iter,
            vao,
            block_distance
        })
    }

    pub fn init_global_buffers(&self) -> Result<(), InitializeErr> {
        use super::{vbo, vao};

        {
            let float_vbo = vbo::VertexBufferObject::new::<f32>(
                vec![
                    self.min_point.x, self.min_point.y, self.min_point.z, 0.0,
                    self.scale,
                    1.0 / self.scale,
                    1.0 / self.cell_count as f32, 
                ],
                gl::ARRAY_BUFFER,
                gl::DYNAMIC_COPY
            );
            let attrib_min = vao::VertexAttributePointer {
                location: 8,
                size: 4, 
                offset: 0
            };
            let attrib_scales = vao::VertexAttributePointer {
                location: 9,
                size: 2, 
                offset: 4
            };
            let attrib_inv_cell_count = vao::VertexAttributePointer {
                location: 10,
                size: 1, 
                offset: 6
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 6, float_vbo.id()); } 
            unsafe { super::check_for_gl_error()?; }
            self.vao.append_vbo::<f32>(vec![attrib_min, attrib_scales, attrib_inv_cell_count], float_vbo.id(), gl::FLOAT);
            unsafe { super::check_for_gl_error()?; }
        }

        {
            let int_vbo = vbo::VertexBufferObject::new::<i32>(
                vec![
                    self.max_depth,
                    self.max_traversal_iter,
                    self.cell_count,
                ],
                gl::ARRAY_BUFFER,
                gl::DYNAMIC_COPY
            );
            let attrib_depth = vao::VertexAttributePointer {
                location: 11,
                size: 1, 
                offset: 0
            };
            let attrib_max_iter = vao::VertexAttributePointer {
                location: 12,
                size: 1, 
                offset: 1
            };
            let attrib_cell_count = vao::VertexAttributePointer {
                location: 13,
                size: 1, 
                offset: 2
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 7, int_vbo.id()); } 
            unsafe { super::check_for_gl_error()?; }
            self.vao.append_vbo::<i32>(vec![attrib_depth, attrib_max_iter, attrib_cell_count], int_vbo.id(), gl::INT);
            unsafe { super::check_for_gl_error()?; }
            
            let atmoic_counter_vbo = vbo::VertexBufferObject::new::<i32>(
                vec![
                    self.active_cell_count
                ],
                gl::ATOMIC_COUNTER_BUFFER,
                gl::DYNAMIC_COPY
            );
            let attrib_active_cell_count = vao::VertexAttributePointer {
                location: 14,
                size: 1, 
                offset: 0
            };
            unsafe { gl::BindBufferBase(gl::ATOMIC_COUNTER_BUFFER, 0,  atmoic_counter_vbo.id()); } 
            unsafe { super::check_for_gl_error()?; }
            self.vao.append_vbo::<i32>(vec![attrib_active_cell_count], atmoic_counter_vbo.id(), gl::INT);
            unsafe { super::check_for_gl_error()?; }
        }

        {
            let update_vbo = vbo::VertexBufferObject::new::<f32>(
                vec![0.0; 1000],
                gl::ARRAY_BUFFER,
                gl::DYNAMIC_COPY
            );
            let attrib_pos = vao::VertexAttributePointer {
                location: 14,
                size: 3, 
                offset: 0
            };
            // let attrib_type = vao::VertexAttributePointer {
            //     location: 15,
            //     size: 1, 
            //     offset: 3
            // };
            // let attrib_value = vao::VertexAttributePointer {
            //     location: 16,
            //     size: 1, 
            //     offset: 4
            // };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 5, update_vbo.id()); } 
            unsafe { super::check_for_gl_error()?; }
            self.vao.append_vbo::<f32>(vec![attrib_pos], update_vbo.id(), gl::INT);
            unsafe { super::check_for_gl_error()?; }
        }

        Ok(())
    }

    pub fn block_distance(&self) -> f32 {
        self.block_distance
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn min_point(&self) -> Vector3<f32> {
        self.min_point
    }

    pub fn point_inside(&self, point: &Vector3<f32>) -> bool {
        point.x >= self.min_point.x && point.y >= self.min_point.y && point.z >= self.min_point.z 
        && point.x <= self.min_point.x + self.scale && point.y <= self.min_point.y + self.scale && point.z <= self.min_point.z + self.scale
    }

    pub fn update_vbo(&self, delta: &Vec::<f32>, len: usize, update_compute: &ComputeShader) {
        const LOCAL_GROUP_SIZE_X: f32 = 32.0 * 32.0;

        let size = (len * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr; // size of data in bytes
        unsafe { gl::BufferSubData(gl::SHADER_STORAGE_BUFFER, 0, size, delta.as_ptr() as *const gl::types::GLvoid); }
        
        let x_schedule = len as f32 * 0.2; 
        let dispatch_count = (len as f32 * 0.2) as i32;
        if (x_schedule / LOCAL_GROUP_SIZE_X).fract() != 0.0 {
            update_compute.dispatch_compute(0, dispatch_count, 0)
        } else {
            update_compute.dispatch_compute(dispatch_count, 1, 1)
        }
    }
}