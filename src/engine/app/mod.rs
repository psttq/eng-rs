pub mod renderer;
pub mod game;
pub mod texture_manager;

use game::{components::Label, GameHandler};
use hecs::{Entity, World};
use renderer::State;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use crate::engine::app::{game::components::{self, Script, TransformComponent}, texture_manager::TextureManager};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    window::{Window, WindowId}
};

use std::time::{Instant};

pub struct GameManager{
    state: Rc<RefCell<State>>,
    pub texture_manager: TextureManager,
    pub world: World
}

impl GameManager{
    fn new(state: Rc<RefCell<State>>) -> Self{
        let texture_manager = TextureManager::new(state.clone());

        Self {
            state,
            texture_manager,
            world: World::new()
        }
    }

    pub fn add_object(&mut self, label: &str) -> hecs::Entity{
        self.world.spawn((Label::from_str(label),))
    }

    pub fn add_components_to_object(&mut self, entity: hecs::Entity, components: impl hecs::DynamicBundle){
        self.world.insert(entity, components).expect("Error while adding components to entity");
    }

    pub fn add_component_to_object(&mut self, entity: hecs::Entity, component: impl hecs::Component){
        self.world.insert_one(entity, component).expect("Error while adding component to entity");
    }

    pub fn get_component_from_object<'a, T: hecs::ComponentRef<'a>>(&mut self, entity: hecs::Entity) -> Option<hecs::Ref<T>>
    where T: hecs::Component
    {
        match self.world.get::<&T>(entity){
            Ok(component) => Some(component),
            _ => None
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

    fn get_dt(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        delta_time.as_secs_f32()
    }
}

impl<T> ApplicationHandler for App<T>
    where T: GameHandler
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = Rc::new(RefCell::new(pollster::block_on(State::new(window.clone()))));
        
        self.game_manager = Some(GameManager::new(state.clone()));
        let gm = self.game_manager.as_mut().unwrap();
        self.game.on_start(gm);
        
        self.state = Some(state);
        self.last_frame_time = Instant::now();

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let dt = self.get_dt();

        let gm = self.game_manager.as_mut().unwrap();
        self.game.update(gm, dt);

        let state = self.state.as_mut().unwrap();
        let mut state = state.borrow_mut();
        state.input(&event);
        state.update(dt);
        
        for (id, script) in &mut gm.world.query::<&mut components::Script>(){
            script.update(dt, &gm.world, &id);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render(|game_mananger, renderer| {self.game.on_ui(game_mananger, renderer);}, gm);
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => (),
        }
    }
}
