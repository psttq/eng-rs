mod renderer;
pub mod game;

use game::GameHandler;
use renderer::State;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    keyboard::PhysicalKey, window::{Window, WindowId}
};

use std::time::{Instant};

pub struct App<T>
    where T: GameHandler
{
    state: Option<State>,
    last_frame_time: Instant,
    game: T
}


impl<T> App<T>
    where T: GameHandler
{
    pub fn new(game: T) -> Self{
        Self { state: None, last_frame_time: Instant::now(), game }
    }
}

impl<T> ApplicationHandler for App<T>
    where T: GameHandler
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        self.last_frame_time = Instant::now();
        self.game.on_start();

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        // Convert to seconds as f32 (common in game engines)
        let delta_time_secs = delta_time.as_secs_f32();
        let state = self.state.as_mut().unwrap();
        state.input(&event);
        state.update(delta_time_secs);
        self.game.update(delta_time_secs);
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } =>{
                if event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::Escape){
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}
