mod engine;
use engine::app::{App, GameManager};
use engine::app::game::GameHandler;
use engine::app::game::components;

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
    fn on_start(&mut self, gm: &mut GameManager) {
        let texture = gm.texture_manager.load_texture("player", "2.png").unwrap();
        let sprite = components::sprite::Sprite{texture};
        self.world.spawn(("Label", sprite));
    }

    fn update(&mut self, gm: &mut GameManager, dt: f32) {
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