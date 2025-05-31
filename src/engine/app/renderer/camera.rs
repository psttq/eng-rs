pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub speed: f32,
    pub aspect: f32,
    pub scale: f32
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::ortho((-1.0*self.aspect)*self.scale  + self.position.x, (1.0*self.aspect)*self.scale  + self.position.x, (-1.0*self.scale) + self.position.y, (1.0*self.scale) + self.position.y, -1.0, 1.0);
        return OPENGL_TO_WGPU_MATRIX * proj;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey}
};

pub struct CameraController {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_plus_pressed: bool,
    is_minus_pressed: bool
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_plus_pressed: false,
            is_minus_pressed: false
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::Equal => {
                        self.is_plus_pressed = is_pressed;
                        true
                    }
                    KeyCode::Minus => {
                        self.is_minus_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {

        if self.is_forward_pressed{
            camera.position.y += camera.speed * dt;
        }
        
        if self.is_backward_pressed{
            camera.position.y -= camera.speed * dt;
        }

        if self.is_left_pressed{
            camera.position.x -= camera.speed * dt;
        }

        if self.is_right_pressed{
            camera.position.x += camera.speed * dt;
        }

        if self.is_plus_pressed{
            camera.scale += 0.01;
        }

        if self.is_minus_pressed{
            camera.scale -= 0.01;
        }
    }
}
