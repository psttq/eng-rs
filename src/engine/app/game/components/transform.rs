use std::sync::{Arc, Mutex};

pub struct Position{
    pub x: f32,
    pub y: f32
}

impl Position{
    fn to_mat(&self) -> cgmath::Matrix4<f32>{
        let position = cgmath::vec3(self.x, self.y, 0.0);
        cgmath::Matrix4::from_translation(position)
    }
}

pub struct Rotation{
    pub angle: f32
}

impl Rotation{
    fn to_mat(&self) -> cgmath::Matrix4<f32>{
        cgmath::Matrix4::from_angle_z(cgmath::Rad(self.angle))
    }
}

pub struct Transform{
    pub position: Position,
    pub rotation: Rotation
}

pub type TransformComponent = Arc<Mutex<Transform>>;

impl Transform{
    pub fn new(x: f32, y: f32, angle: f32) -> TransformComponent{
        Arc::new(Mutex::new(Self{ position: Position { x: x, y: y }, rotation: Rotation { angle:  angle}}))
    }

    pub fn to_mat(&self) -> cgmath::Matrix4<f32>{
        self.position.to_mat() * self.rotation.to_mat()
    }
}