use cgmath::Vector4;
use std::string::String;

#[repr(C)]
pub struct Pixel {
    /// used to select proper pixel struct from a loaded texture
    alpha_id: f64,
    /// display color of pixel type
    color: Vector4<f32>,
    /// logic to update any pixel, currently raw glsl injected into main shader at runtime
    update_function: String,
}

impl Pixel {
    // TODO: proper error handling for failing 
    #[allow(dead_code)]
    pub fn new(alpha_id: f64, color: Vector4<f32>, update_function: String) -> Result<Pixel, &'static str> {
        if !is_valid_update_function(&update_function) {
            return Err("failed to validate update function");
        }

        Ok(Pixel {
            alpha_id,
            color,
            update_function,
        })
    }

    #[allow(dead_code)]
    pub fn alpha_id(&self) -> f64 {
        self.alpha_id
    }

    #[allow(dead_code)]
    pub fn color(&self) -> Vector4<f32> {
        self.color
    }

    #[allow(dead_code)]
    pub fn update_function(&self) -> &str {
        self.update_function.as_ref()
    }
}

// TODO:
#[allow(dead_code)]
fn is_valid_update_function(update_function: &str) -> bool {
    true
}