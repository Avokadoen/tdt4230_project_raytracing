use cgmath::Vector3;

pub mod chronos;

pub enum Direction {
    Front,
    Back,
    Rigth,
    Left,
    Up,
    Down,
}

impl Direction {
    pub fn into_vector3(self) -> Vector3::<f32> {
        match self {
            Direction::Front => Vector3::<f32>::new( 0.0,  0.0, -1.0),
            Direction::Back  => Vector3::<f32>::new( 0.0,  0.0,  1.0),
            Direction::Rigth => Vector3::<f32>::new( 1.0,  0.0,  0.0),
            Direction::Left  => Vector3::<f32>::new(-1.0,  0.0,  0.0),
            Direction::Up    => Vector3::<f32>::new( 0.0,  1.0,  0.0),
            Direction::Down  => Vector3::<f32>::new( 0.0, -1.0,  0.0),
        }
    }
}