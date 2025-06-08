use crate::engine::app::GameManager;
use crate::engine::app::renderer::egui_tools::EguiRenderer;
pub mod components;
pub trait GameHandler
{
    fn on_start(&mut self, gm: &mut GameManager);
    fn update(&mut self, gm: &mut GameManager, dt: f32);
    fn on_ui(&mut self, gm: &mut GameManager, egui_renderer: &EguiRenderer);
}