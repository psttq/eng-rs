use std::{env, fs, path::PathBuf, sync::Arc};

use mlua::prelude::*;
use hecs::{Entity, World};

use crate::engine::app::game::components::{script, transform::TransformComponent};

use log::{error};

pub enum ScriptState{
    Ok,
    Err(String)
}

pub struct Script{
    path: PathBuf,
    script: String,
    pub state: ScriptState,
    pub lua: Arc<Lua>,
}

impl Script{
    pub fn new(path: String) -> Self{
         let exe_path = env::current_exe().unwrap();
    
        // Get the directory containing the executable
        let exe_dir = exe_path.parent()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::Other,
                "Executable has no parent directory"
            )).unwrap();
        
        // Construct path to your file relative to the executable
        let file_path = exe_dir.join(path);
        let script = fs::read_to_string(&file_path).unwrap();
        let lua = Lua::new();
        match lua.load(script.clone()).exec(){
            Ok(()) => {},
            Err(e) => {
                error!("{:?}", e);
            }
        }
        Self {path: file_path, script: script, state: ScriptState::Ok, lua: Arc::new(lua) }
    }

    pub fn reload(&mut self){
        let script = fs::read_to_string(&self.path).unwrap();
        self.set_script(script);
    }

    pub fn set_script(&mut self, script: String){
        self.script = script.clone();
        match self.lua.load(script.clone()).exec(){
            Ok(()) => {
                self.state = ScriptState::Ok;
            },
            Err(e) => {
                error!("{:?}", e);
                self.state = ScriptState::Err(e.to_string());
            }
        }

        fs::write(&self.path, self.script.clone()).unwrap();
    }

    pub fn get_script(&self) -> String{
        self.script.clone()
    }

    pub fn update(&mut self, dt:f32, world: &World, entity: &Entity){
        let result = self.lua.scope(|scope|{
            let game_object_table = self.lua.create_table().unwrap();
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

            self.lua.globals().set("gameObject", game_object_table).unwrap();

            let update_func = match self.lua.globals().get::<mlua::Function>("update"){
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