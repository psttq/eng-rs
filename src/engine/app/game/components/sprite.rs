use std::sync::Arc;

use crate::engine::app::renderer::texture::Texture;

pub struct Sprite{
    pub texture: Arc<Texture>
}

impl Sprite{
    pub fn new(texture: Arc<Texture>) -> Self{
        Self{texture}
    }
}