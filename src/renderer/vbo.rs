use gl::types::GLenum;

pub struct VertexBufferObject {
    id: u32,
    length: i32,
    binding: GLenum
}

impl VertexBufferObject {

    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[allow(dead_code)]
    pub fn length(&self) -> i32 {
        self.length
    }

    #[allow(dead_code)]
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.binding, self.id);
        }
    }
    
    #[allow(dead_code)]
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.binding, 0);
        }
    }

    // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glBindBuffer.xhtml
    #[allow(dead_code)]
    pub fn new<T>(vertex_buffer: Vec<T>, binding: GLenum) -> Self{
        let mut id: gl::types::GLuint = 0;
        let length = vertex_buffer.len() as i32;
        let binding = binding;
        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(binding, id);

            gl::BufferData(
                binding, // target
                (vertex_buffer.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertex_buffer.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindBuffer(binding, 0);
        }

        VertexBufferObject {
            id,
            length,
            binding
        }
    }
}