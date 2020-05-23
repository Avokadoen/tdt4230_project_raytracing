use cgmath::Vector2;
pub enum Direction {
    Up,
    Rigth,
    Down,
    Left
}

impl Direction {
    pub fn into_vector2(self) -> Vector2::<f32> {
        match self {
            Direction::Up =>    Vector2::<f32>::new(0.0, -1.0),
            Direction::Rigth => Vector2::<f32>::new(-1.0, 0.0),
            Direction::Down =>  Vector2::<f32>::new(0.0, 1.0),
            Direction::Left =>  Vector2::<f32>::new(1.0, 0.0)
        }
    }
}