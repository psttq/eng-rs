pub trait GameHandler{
    fn on_start(&self);
    fn update(&self, dt: f32);
}