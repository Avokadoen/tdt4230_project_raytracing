use cgmath::Vector3;

use crate::renderer::texture::Texture;

use super::program::Program;

// TODO: camera can probably hold and create render texture to avoid programatic errors
// TODO: camera should have a say when it comes to viewport and program window
pub struct Camera {
    pub horizontal: Vector3::<f32>,
    pub vertical: Vector3::<f32>,
    
    pub lower_left_corner: Vector3::<f32>,
    pub origin: Vector3::<f32>,
    
    pub image_width: i32,
    pub image_height: i32,
    pub render_texture: Texture,
}

impl Camera {
    // TODO: move camera, set origin and rotation when we get so far :)
}

pub struct CameraBuilder {
    image_width: i32,
    focal_length: Option<f32>,
    aspect_ratio: Option<f32>,
    viewport_height: Option<f32>,
    origin: Option<Vector3::<f32>>,
}

impl CameraBuilder {
    pub fn new(image_width: i32) -> CameraBuilder {
        // create the camera builder with defaults
        CameraBuilder {
            image_width,
            focal_length: None,
            aspect_ratio: None,
            viewport_height: None,
            origin: None,
        }
    }

    pub fn build(&mut self, program: &mut Program) -> Camera {
        let aspect_ratio = self.aspect_ratio.unwrap_or(16.0 / 9.0);
        let viewport_height = self.viewport_height.unwrap_or(2.0);
        let viewport_width = aspect_ratio * viewport_height;
        
        let origin = self.origin.unwrap_or(Vector3::new(0.0, 0.0, 0.0));
        
        let focal_length = self.focal_length.unwrap_or(1.0);
        let horizontal = Vector3::<f32>::new(viewport_width, 0.0, 0.0);
        let vertical = Vector3::<f32>::new(0.0, viewport_height, 0.0);
        let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - Vector3::<f32>::new(0.0, 0.0, focal_length);

        let image_height = (self.image_width as f32 / aspect_ratio) as i32;
        let camera = Camera {
            horizontal,
            vertical,
            lower_left_corner,
            origin,
            image_width: self.image_width,
            image_height,
            render_texture: Texture::new( 
                gl::TEXTURE0, 
                0, 
                gl::RGBA32F, 
                gl::RGBA, 
                self.image_width, 
                image_height
            ),
        };

        initial_uniforms(&camera, program);

        return camera;
    }

    pub fn with_aspect_ratio(&mut self, aspect_ratio: f32) -> &mut CameraBuilder {
        self.aspect_ratio = Some(aspect_ratio);
        return self;
    } 

    pub fn with_viewport_height(&mut self, viewport_height: f32) -> &mut CameraBuilder {
        self.viewport_height = Some(viewport_height);
        return self;
    }

    pub fn with_origin(&mut self, origin: Vector3::<f32>) -> &mut CameraBuilder {
        self.origin = Some(origin);
        return self;
    }
}


// Sets all camera variables in the shader 
fn initial_uniforms(camera: &Camera, program: &mut Program) {
    // TODO: don't unwrap ... 
    program.set_i32("camera.image_width", camera.image_width).unwrap();
    program.set_i32("camera.image_height", camera.image_height).unwrap();
    
    program.set_vector3_f32("camera.horizontal", camera.horizontal).unwrap();
    program.set_vector3_f32("camera.vertical", camera.vertical).unwrap();

    program.set_vector3_f32("camera.lower_left_corner", camera.lower_left_corner).unwrap();
    program.set_vector3_f32("camera.origin", camera.origin).unwrap();
}