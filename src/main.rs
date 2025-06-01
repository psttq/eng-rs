mod engine;
use engine::app::{App, GameManager};
use engine::app::game::GameHandler;
use engine::app::game::components;
use engine::app::renderer::egui_tools::EguiRenderer;

use hecs::World;

use winit::{
    event_loop::{ControlFlow, EventLoop},
};

struct Game{
}

impl Game{
    fn new() -> Self {
        Self { }
    }
}

impl GameHandler for Game{
    fn on_start(&mut self, gm: &mut GameManager) {
        let texture = gm.texture_manager.load_texture("player", "2.png").unwrap();
        let sprite = components::Sprite::new(texture);
        let player = gm.add_object("Player");
        gm.add_component_to_object(player, sprite);
        gm.add_component_to_object(player, components::Position::new(0.0, 0.0));
    }

    fn update(&mut self, gm: &mut GameManager, dt: f32) {
        
    }

    fn on_ui(&mut self, egui_renderer: &EguiRenderer) {
        egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(egui_renderer.context(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            egui_renderer.context().pixels_per_point()
                        ));
                    });
                });
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