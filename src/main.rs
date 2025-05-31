mod engine;
use engine::app::App;
use engine::app::game::GameHandler;

use hecs::World;

use winit::{
    event_loop::{ControlFlow, EventLoop},
};

struct Game{
    world: World
}

impl Game{
    fn new() -> Self {
        Self { world: World::new() }
    }
}

impl GameHandler for Game{
    fn on_start(&self) {
        
    }

    fn update(&self, dt: f32) {
    }
}

fn main() {
    let game = Game::new();


    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app: App<Game> = App::new(game);
    event_loop.run_app(&mut app).unwrap();
}