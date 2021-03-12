use gl::types::{
    GLuint,
    GLint,
};

#[allow(dead_code)]
pub struct VertexAttributePointer {
    pub location: GLuint,
    pub size: GLint,
    pub offset: usize
}

#[allow(dead_code)]
pub struct VertexArrayObject {
    id: u32,
}

impl VertexArrayObject {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[allow(dead_code)]
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    // TODO: should this be part of impl?
    #[allow(dead_code)]
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    // TODO: VBO type
    #[allow(dead_code)]
    pub fn new(attributes: Vec<VertexAttributePointer>, components: usize, vbo: u32) -> Self {
        let mut id: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut id);
    
            gl::BindVertexArray(id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let stride = (components * std::mem::size_of::<f32>()) as gl::types::GLint;

            for attribute in attributes {
                gl::EnableVertexAttribArray(attribute.location);

                gl::VertexAttribPointer(
                    attribute.location,     // index of the generic vertex attribute ("layout (location = 0)")
                    attribute.size,         // the number of components per generic vertex attribute
                    gl::FLOAT,              // data type
                    gl::FALSE,              // normalized (int-to-float conversion)
                    stride,                 // stride (byte offset between consecutive attributes)
                    (attribute.offset * std::mem::size_of::<f32>()) as *const gl::types::GLvoid    // offset of the first component
                );
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        VertexArrayObject {
            id
        }
    }
}