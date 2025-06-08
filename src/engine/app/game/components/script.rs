use mlua::prelude::*;
use hecs::{Entity, World};

use crate::engine::app::game::components::{transform::TransformComponent};

use log::{error};

pub enum ScriptState{
    Ok,
    Err(String)
}

pub struct Script{
    pub script: String,
    pub state: ScriptState
}

impl Script{
    pub fn new(script: String) -> Self{
        Self { script: script, state: ScriptState::Ok }
    }
    pub fn update(&mut self, dt:f32, world: &World, entity: &Entity){
        let lua = Lua::new();
        let result = lua.scope(|scope|{
            let game_object_table = lua.create_table().unwrap();
            match world.get::<&TransformComponent>(*entity){
                Ok(transform_arc) => {
                    let transform_arc_clone = transform_arc.clone();
                    let get_position_func = scope.create_function( move |_,()|{
                        let transform = transform_arc_clone.lock().unwrap();
                        Ok((transform.position.x, transform.position.y))
                    }).unwrap();
                    game_object_table.set("getPosition", get_position_func).unwrap();

                    let transform_arc_clone = transform_arc.clone();

                    let set_position_func = scope.create_function( move |_,(x,y):(f32, f32)|{
                        let mut transform = transform_arc_clone.lock().unwrap();
                        transform.position.x = x;
                        transform.position.y = y;
                        Ok(())
                    }).unwrap();
                    game_object_table.set("setPosition", set_position_func).unwrap();
                },
                _ => {}
            }

            lua.globals().set("gameObject", game_object_table).unwrap();

            match lua.load(self.script.clone()).exec(){
                Ok(()) => {},
                Err(e) => {
                    error!("{:?}", e);
                }
            }
            let update_func = match lua.globals().get::<mlua::Function>("update"){
                Ok(func)=>func,
                Err(e) => {
                    self.state = ScriptState::Err(e.to_string());
                    return Err(e);
                }
            };

            update_func.call::<()>(dt)
        });

        match result{
            Ok(()) => self.state = ScriptState::Ok,
            Err(e) => self.state = ScriptState::Err(e.to_string())
        }
    }
}