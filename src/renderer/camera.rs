use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};

use crate::renderer::texture::Texture;

use super::{InitializeErr, program::Program};

// TODO: camera should have a say when it comes to viewport and program window
// TODO: some of the cameras variables can be remove as they are only used when 
//       they are calculated (horizontal, vertical, lower_left_corner ...)
pub struct Camera {
    pub horizontal: Vector3::<f32>,
    pub vertical: Vector3::<f32>,
    pub viewport_width: f32,
    pub viewport_height: f32,

    pub lower_left_corner: Vector3::<f32>,
    pub origin: Vector3::<f32>,
    pitch: Quaternion<f32>,
    yaw: Quaternion<f32>,
    
    // TODO: these are only i32 because it is easier to send to GPU
    pub samples_per_pixel: i32,
    pub max_bounce: i32,

    pub image_width: i32,
    pub image_height: i32,
    pub render_texture: Texture,

    pub turn_rate: f32,
    pub movement_speed: f32,
}

impl Camera {
    pub fn translate(&mut self, program: &mut Program, by: &Vector3::<f32>, deltatime: f64) {
        self.origin += self.orientation().rotate_vector(*by * deltatime as f32 * self.movement_speed);
        self.propagate_changes(program);
    }
     
    // turn in x axis
    pub fn turn_pitch(&mut self, program: &mut Program, angle: f32) {
        // Axis angle to quaternion: https://www.euclideanspace.com/maths/geometry/rotations/conversions/angleToQuaternion/index.htm
        let h_angle = angle * self.turn_rate;
        let i = h_angle.sin();
        let w = h_angle.cos();
        self.pitch = self.pitch * Quaternion::new(w, i, 0.0, 0.0).normalize();
        self.propagate_changes(program);
    }

    // turn in y axis
    pub fn turn_yaw(&mut self, program: &mut Program, angle: f32) {
        let h_angle = angle * self.turn_rate;
        let j = h_angle.sin();
        let w = h_angle.cos();
        self.yaw = self.yaw * Quaternion::new(w, 0.0, j, 0.0).normalize();
        self.propagate_changes(program);
    }

    fn orientation(&self) -> Quaternion<f32> {
        return (self.yaw * self.pitch).normalize();
    }

    fn propagate_changes(&mut self, program: &mut Program) {
        let forward = (self.orientation().rotate_vector(Vector3::unit_z())).normalize();
        let right = Vector3::unit_y().cross(forward).normalize();
        let up = forward.cross(right).normalize();

        self.horizontal = right * self.viewport_width;
        self.vertical = up * self.viewport_height;
        self.lower_left_corner = self.origin - self.horizontal * 0.5 - self.vertical * 0.5 - forward;

        // TODO: only set what has changed
        program.set_vector3_f32("camera.horizontal", self.horizontal).unwrap();
        program.set_vector3_f32("camera.vertical", self.vertical).unwrap();
        program.set_vector3_f32("camera.lower_left_corner", self.lower_left_corner).unwrap();
        program.set_vector3_f32("camera.origin", self.origin).unwrap();
    }

    pub fn set_movement_speed(&mut self, movement_speed: f32) {
        self.movement_speed = movement_speed;
    }
}

// TODO: with pitch, yaw
pub struct CameraBuilder {
    vertical_fov: f32,
    image_width: i32,
    aspect_ratio: Option<f32>,
    viewport_height: Option<f32>,
    origin: Option<Vector3::<f32>>,
    samples_per_pixel: Option<i32>,
    max_bounce: Option<i32>,
    turn_rate: Option<f32>,
    movement_speed: Option<f32>,
}

impl CameraBuilder {
    pub fn new(vertical_fov: f32, image_width: i32) -> CameraBuilder {
        // create the camera builder with defaults
        CameraBuilder {
            vertical_fov,
            image_width,
            aspect_ratio: None,
            viewport_height: None,
            origin: None,
            samples_per_pixel: None,
            max_bounce: None,
            turn_rate: None,
            movement_speed: None,
        }
    }

    pub fn build(&mut self, program: &mut Program) -> Result<Camera, InitializeErr> {
        let aspect_ratio = self.aspect_ratio.unwrap_or(16.0 / 9.0);

        let theta = self.vertical_fov * std::f32::consts::PI / 180.0;
        let h = (theta / 2.0).tan();
        let viewport_height = self.viewport_height.unwrap_or(2.0) * h;
        let viewport_width = aspect_ratio * viewport_height;
        
        let origin = self.origin.unwrap_or(Vector3::new(0.0, 0.0, 0.0));
        
        let forward = (Vector3::unit_z()).normalize();
        let right = Vector3::unit_y().cross(forward).normalize();
        let up = forward.cross(right).normalize();

        let horizontal: Vector3<f32> = right * viewport_width;
        let vertical: Vector3<f32> = up * viewport_height;
        
        let lower_left_corner = origin - horizontal * 0.5 - vertical * 0.5 - forward;

        let image_height = (self.image_width as f32 / aspect_ratio) as i32;

        let sample_per_pixel = self.samples_per_pixel.unwrap_or(10);
        let max_bounce = self.max_bounce.unwrap_or(3);
        let render_texture = Texture::new_2d( 
            gl::TEXTURE0, 
            0, 
            gl::RGBA32F, 
            gl::RGBA, 
            self.image_width, 
            image_height
        )?;

        let turn_rate = self.turn_rate.unwrap_or(0.025);
        let movement_speed = self.movement_speed.unwrap_or(1.0);

        let camera = Camera {
            horizontal,
            vertical,
            viewport_width,
            viewport_height,
            lower_left_corner,
            origin,
            pitch: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            yaw: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            image_width: self.image_width,
            image_height,
            render_texture,
            samples_per_pixel: sample_per_pixel,
            max_bounce,
            turn_rate,
            movement_speed,
        };

        initial_uniforms(&camera, program);

        return Ok(camera);
    }

    pub fn with_turn_rate(&mut self, turn_rate: f32) -> &mut CameraBuilder {
        self.turn_rate = Some(turn_rate);
        return self;
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