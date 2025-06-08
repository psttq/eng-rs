mod engine;
use egui::Ui;
use engine::app::{App, GameManager};
use engine::app::game::GameHandler;
use engine::app::game::components;
use engine::app::renderer::egui_tools::EguiRenderer;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};

use hecs::Entity;
use winit::{
    event_loop::{ControlFlow, EventLoop},
};

use crate::engine::app::game::components::{Script, ScriptState, Sprite, TransformComponent};

struct Game{
    player: Option<Entity>
}

impl Game{
    fn new() -> Self {
        Self {player: None}
    }
}

impl GameHandler for Game{
    fn on_start(&mut self, gm: &mut GameManager) {
        let texture = gm.texture_manager.load_texture("player", "2.png").unwrap();
        let sprite = components::Sprite::new(texture.clone());
        let player = gm.add_object("Player");
        gm.add_component_to_object(player, sprite);
        gm.add_component_to_object(player, components::Transform::new(0.0, 0.0, 30.0));

        let sprite = components::Sprite::new(texture.clone());
        let player = gm.add_object("Player 3");
        gm.add_component_to_object(player, sprite);
        gm.add_component_to_object(player, components::Transform::new(-1.0, 0.0, 0.0));


        let texture = gm.texture_manager.load_texture("happy-tree", "happy-tree.png").unwrap();
        let sprite = components::Sprite::new(texture);
        let script = components::Script::new("function update(dt)
    local x, y = gameObject.getPosition()
    gameObject.setPosition(x + 0.1*dt, y + 0.1*dt)
end".to_string());
        let player = gm.add_object("Player 2");
        gm.add_component_to_object(player, sprite);
        gm.add_component_to_object(player, script);
        gm.add_component_to_object(player, components::Transform::new(1.0, 1.0, 0.0));
        self.player = Some(player);
    }

    fn update(&mut self, _gm: &mut GameManager, _dt: f32) {

    }

    fn on_ui(&mut self, gm: &mut GameManager, egui_renderer: &mut EguiRenderer) {
       
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