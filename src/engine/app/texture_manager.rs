use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::{collections::HashMap};
use crate::engine::app::renderer::State;

use crate::engine::app::renderer::texture::Texture;
pub struct TextureManager{
    textures: HashMap<String, Arc<Texture>>,
    state: Rc<RefCell<State>>
}

impl TextureManager{
    pub fn new(state: Rc<RefCell<State>>) -> Self{
        Self{
            textures: HashMap::default(),
            state
        }
    }

    pub fn load_texture(&mut self, name: &str, path: &str) -> Option<Arc<Texture>>{
        let state = self.state.borrow_mut();
       let texture = state.load_texture(name, path);
       self.textures.insert(name.to_string(), Arc::new(texture));
       self.textures.get(name).cloned()
    }

    pub fn get_texture(&self, name: &str) -> Option<Arc<Texture>>{
        self.textures.get(name).cloned()
    }
}