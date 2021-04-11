use gl::types::{GLenum, GLint, GLuint};

pub struct VertexAttributePointer {
    pub location: GLuint,
    pub size: GLint,
    pub offset: usize
}

pub struct VertexArrayObject {
    id: u32,
}

impl VertexArrayObject {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    // TODO: components can be retrieved from attributes
    pub fn new<T>(attributes: Vec<VertexAttributePointer>, vbo: u32, buffer_type: GLenum) -> Self {
        let mut id: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        let vao = VertexArrayObject {
            id
        };
        vao.append_vbo::<T>(attributes, vbo, buffer_type);

        vao
    }

    pub fn append_vbo<T>(&self, attributes: Vec<VertexAttributePointer>, vbo: u32, buffer_type: GLenum) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
           
            let components = match attributes.iter().max_by(|a, b| a.location.cmp(&b.location)) {
                Some(a) => a.offset + a.size as usize,
                None => 0, // TODO: ERROR
            };
            let stride = (components * std::mem::size_of::<f32>()) as gl::types::GLint;

            for attribute in attributes {
                gl::EnableVertexAttribArray(attribute.location);
                // TODO: AttribI
                gl::VertexAttribPointer(
                    attribute.location,     // index of the generic vertex attribute ("layout (location = 0)")
                    attribute.size,         // the number of components per generic vertex attribute
                    buffer_type,              // data type
                    gl::FALSE,              // normalized (int-to-float conversion)
                    stride,                 // stride (byte offset between consecutive attributes)
                    (attribute.offset * std::mem::size_of::<f32>()) as *const gl::types::GLvoid    // offset of the first component
                );
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

    }
}