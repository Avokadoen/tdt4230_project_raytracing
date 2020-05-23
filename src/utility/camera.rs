use cgmath::{ Vector2, Vector3};
use cgmath::InnerSpace;

use super::direction::Direction;

// TODO: just use this https://docs.rs/cgmath/0.17.0/cgmath/fn.ortho.html
//       It's also disingenuous to call it position as z is actually scale ...
// TODO: this should maybe use f64
pub struct Camera2D {
    /// change in zoom after 1 second of active zooming
    zoom_speed: f32,
    zoom_velocity: f32,
    pan_speed: f32,
    pan_direction: Vector2<f32>,
    position: Vector3<f32>,
}

impl Default for Camera2D {
    fn default() -> Camera2D {
        Self {
            zoom_speed: 6.0,
            zoom_velocity: 0.0,
            pan_speed: 3.0,
            pan_direction: Vector2::new(0.0, 0.0),
            position: Vector3::new(0.0, 0.0, 1.0),
        }
    }
}

impl Camera2D {
    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    // TODO: direction can only be 1.0 or -1.0 find a way to enforce this
    pub fn modify_zoom(&mut self, delta_time: f64, direction: f32) {
        self.zoom_velocity += delta_time as f32 * self.zoom_speed * direction;
    }

    pub fn pan_in_direction(&mut self, direction: Direction) {
        let direction_vec = direction.into_vector2();

        self.pan_direction.x += direction_vec.x;
        self.pan_direction.y += direction_vec.y;
        self.pan_direction = self.pan_direction.normalize();
    }

    // TODO: I have this interface, but i'm not sure how I should actually solve this atm
    /// Applies changes to camera, return true if there were any changes
    pub fn commit_pan_zoom(&mut self, delta_time: f64) -> bool {
        let mut changed = false;
        let delta_time = delta_time as f32;

        if self.zoom_velocity.abs() > 0.00001 {
            changed = true;
            self.position.z += self.zoom_velocity;
            self.zoom_velocity = 0.0;
        }

        if self.pan_direction.x.abs() > 0.00001 {
            changed = true;
            self.position.x += self.pan_direction.x * self.pan_speed * delta_time;
            self.pan_direction.x = 0.0;
        }
        
        if self.pan_direction.y.abs() > 0.00001 {
            changed = true;
            self.position.y += self.pan_direction.y * self.pan_speed * delta_time;
            self.pan_direction.y = 0.0;
        } 

        return changed;
    }
}