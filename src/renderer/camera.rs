use cgmath::{InnerSpace, Vector3};

use crate::renderer::texture::Texture;

use super::program::Program;

// TODO: camera can probably hold and create render texture to avoid programatic errors
// TODO: camera should have a say when it comes to viewport and program window
pub struct Camera {
    pub horizontal: Vector3::<f32>,
    pub vertical: Vector3::<f32>,
    pub viewport_width: f32,
    pub viewport_height: f32,
    
    pub lower_left_corner: Vector3::<f32>,
    pub view_dir: Vector3::<f32>,
    pub up_dir: Vector3::<f32>,
    pub origin: Vector3::<f32>,
    
    // TODO: these are only i32 because it is easier to send to GPU
    pub samples_per_pixel: i32,
    pub max_bounce: i32,

    pub image_width: i32,
    pub image_height: i32,
    pub render_texture: Texture,
}

impl Camera {
    // TODO: move camera, set origin and rotation when we get so far :)
    pub fn translate(&mut self, by: &Vector3::<f32>, deltatime: f64, program: &mut Program) {
        self.origin += *by * deltatime as f32;

        let w = (-self.view_dir).normalize();
        let u = self.up_dir.cross(w).normalize();
        let v = w.cross(u);

        let horizontal = u * self.viewport_width;
        let vertical = v * self.viewport_height;
        self.lower_left_corner = self.origin - horizontal/2.0 - vertical/2.0 - w;

        program.set_vector3_f32("camera.lower_left_corner", self.lower_left_corner).unwrap();
        program.set_vector3_f32("camera.origin", self.origin).unwrap();
    }
}

pub struct CameraBuilder {
    vertical_fov: f32,
    image_width: i32,
    aspect_ratio: Option<f32>,
    viewport_height: Option<f32>,
    view_dir: Option<Vector3::<f32>>,
    up_dir: Option<Vector3::<f32>>,
    origin: Option<Vector3::<f32>>,
    samples_per_pixel: Option<i32>,
    max_bounce: Option<i32>,

}

impl CameraBuilder {
    pub fn new(vertical_fov: f32, image_width: i32) -> CameraBuilder {
        // create the camera builder with defaults
        CameraBuilder {
            vertical_fov,
            image_width,
            aspect_ratio: None,
            viewport_height: None,
            view_dir: None,
            up_dir: None,
            origin: None,
            samples_per_pixel: None,
            max_bounce: None,
        }
    }

    pub fn build(&mut self, program: &mut Program) -> Camera {
        let aspect_ratio = self.aspect_ratio.unwrap_or(16.0 / 9.0);

        let theta = self.vertical_fov * std::f32::consts::PI / 180.0;
        let h = (theta / 2.0).tan();
        let viewport_height = self.viewport_height.unwrap_or(2.0) * h;
        let viewport_width = aspect_ratio * viewport_height;
        
        let view_dir = self.view_dir.unwrap_or(Vector3::new(0.0, 0.0, 1.0));
        let up_dir = self.up_dir.unwrap_or(Vector3::new(0.0, 1.0, 0.0));
        let origin = self.origin.unwrap_or(Vector3::new(0.0, 0.0, 0.0));
        
        let w = (origin - view_dir).normalize();
        let u = up_dir.cross(w).normalize();
        let v = w.cross(u);

        let horizontal = u * viewport_width;
        let vertical = v * viewport_height;
        let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - w;

        let image_height = (self.image_width as f32 / aspect_ratio) as i32;

        let sample_per_pixel = self.samples_per_pixel.unwrap_or(10);
        let max_bounce = self.max_bounce.unwrap_or(3);

        let camera = Camera {
            horizontal,
            vertical,
            viewport_width,
            viewport_height,
            lower_left_corner,
            view_dir,
            up_dir,
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
            samples_per_pixel: sample_per_pixel,
            max_bounce
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

    pub fn with_view_dir(&mut self, view_dir: Vector3::<f32>) -> &mut CameraBuilder {
        self.view_dir = Some(view_dir);
        return self;
    }

    pub fn with_up_dir(&mut self, up_dir: Vector3::<f32>) -> &mut CameraBuilder {
        self.up_dir = Some(up_dir);
        return self;
    }

    pub fn with_origin(&mut self, origin: Vector3::<f32>) -> &mut CameraBuilder {
        self.origin = Some(origin);
        return self;
    }

    pub fn with_sample_per_pixel(&mut self, sample_per_pixel: i32) -> &mut CameraBuilder {
        self.samples_per_pixel = Some(sample_per_pixel);
        return self;
    }

    pub fn with_max_bounce(&mut self, max_bounce: i32) -> &mut CameraBuilder {
        self.max_bounce = Some(max_bounce);
        return self;
    }
}


// Sets all camera variables in the shader 
fn initial_uniforms(camera: &Camera, program: &mut Program) {
    // TODO: don't unwrap ... (send error to caller) 
    program.set_i32("camera.image_width", camera.image_width).unwrap();
    program.set_i32("camera.image_height", camera.image_height).unwrap();
    
    program.set_vector3_f32("camera.horizontal", camera.horizontal).unwrap();
    program.set_vector3_f32("camera.vertical", camera.vertical).unwrap();

    program.set_vector3_f32("camera.lower_left_corner", camera.lower_left_corner).unwrap();
    program.set_vector3_f32("camera.origin", camera.origin).unwrap();
    
    program.set_i32("camera.samples_per_pixel", camera.samples_per_pixel).unwrap();
    program.set_i32("camera.max_bounce", camera.max_bounce).unwrap();
}