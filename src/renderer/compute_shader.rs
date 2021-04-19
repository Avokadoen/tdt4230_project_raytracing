use gl::types::{GLint};


use super::{InitializeErr, check_for_gl_error, program::Program};



// Compute shader that is bound to screen size. 
pub struct ComputeShader {
    pub program: Program,
    group_size: [GLint; 3]
}

impl ComputeShader {
    pub fn new(program: Program) -> Result<Self, InitializeErr> {
        let mut group_size: [i32; 3] = [0; 3];
        unsafe {
            gl::GetProgramiv(program.id(), gl::COMPUTE_WORK_GROUP_SIZE, group_size.as_mut_ptr());
            check_for_gl_error()?; 
        }  

        Ok(Self {
            program,
            group_size
        })
    }

    pub fn dispatch_compute(&self, width: i32, height: i32, depth: i32) {
        self.program.bind();
        let num_groups_x = ((width  / self.group_size[0]) as u32).max(1);
        let num_groups_y = ((height / self.group_size[1]) as u32).max(1);
        let num_groups_z = ((depth  / self.group_size[2]) as u32).max(1);
        unsafe {
            gl::DispatchCompute(num_groups_x, num_groups_y, num_groups_z);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT); 
        }
        Program::unbind();
    }
}