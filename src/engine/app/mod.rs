mod renderer;
pub mod game;
pub mod texture_manager;

use game::GameHandler;
use hecs::Ref;
use renderer::State;
use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};
use crate::engine::app::texture_manager::TextureManager;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    keyboard::PhysicalKey, window::{Window, WindowId}
};

use std::time::{Instant};

pub struct GameManager{
    state: Rc<RefCell<State>>,
    pub texture_manager: TextureManager,
}

impl GameManager{
    fn new(state: Rc<RefCell<State>>) -> Self{
        let texture_manager = TextureManager::new(state.clone());

        Self {
            state,
            texture_manager
        }
    }
}

pub struct App<T>
    where T: GameHandler
{
    state: Option<Rc<RefCell<State>>>,
    last_frame_time: Instant,
    game: T,
    game_manager: Option<GameManager>
}


impl<T> App<T>
    where T: GameHandler
{
    pub fn new(game: T) -> Self{
        Self { state: None, game_manager: None, last_frame_time: Instant::now(), game }
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

        let state = Rc::new(RefCell::new(pollster::block_on(State::new(window.clone()))));
        self.game_manager = Some(GameManager::new(state.clone()));
        self.state = Some(state);
        self.last_frame_time = Instant::now();
        let gm = self.game_manager.as_mut().unwrap();
        self.game.on_start(gm);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        // Convert to seconds as f32 (common in game engines)
        let delta_time_secs = delta_time.as_secs_f32();

        let gm = self.game_manager.as_mut().unwrap();
        self.game.update(gm, delta_time_secs);

        let state = self.state.as_mut().unwrap();
        let mut state = state.borrow_mut();
        state.input(&event);
        state.update(delta_time_secs);
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
