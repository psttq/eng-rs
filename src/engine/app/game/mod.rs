use crate::engine::app::GameManager;
pub mod components;
pub trait GameHandler
{
    fn on_start(&mut self, app: &mut GameManager);
    fn update(&mut self, app: &mut GameManager, dt: f32);
}