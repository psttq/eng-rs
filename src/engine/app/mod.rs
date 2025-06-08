pub mod renderer;
pub mod game;
pub mod texture_manager;

use egui::{Color32, Frame};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use game::{components::Label, GameHandler};
use hecs::{Entity, World};
use renderer::State;
use std::{cell::RefCell, fmt::format, rc::Rc, sync::Arc};
use crate::engine::app::{game::components::{self, Script, TransformComponent}, renderer::egui_tools::EguiRenderer, texture_manager::TextureManager};

use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::{ElementState, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, window::{Window, WindowId}
};

use std::time::{Instant};

pub struct GameManager{
    state: Rc<RefCell<State>>,
    pub texture_manager: TextureManager,
    pub world: World
}

impl GameManager{
    fn new(state: Rc<RefCell<State>>) -> Self{
        let texture_manager = TextureManager::new(state.clone());

        Self {
            state,
            texture_manager,
            world: World::new()
        }
    }

    pub fn add_object(&mut self, label: &str) -> hecs::Entity{
        self.world.spawn((Label::from_str(label),))
    }

    pub fn add_components_to_object(&mut self, entity: hecs::Entity, components: impl hecs::DynamicBundle){
        self.world.insert(entity, components).expect("Error while adding components to entity");
    }

    pub fn add_component_to_object(&mut self, entity: hecs::Entity, component: impl hecs::Component){
        self.world.insert_one(entity, component).expect("Error while adding component to entity");
    }
}

struct ScriptEditting{
    entity: Entity,
    script: String
}

pub struct App<T>
    where T: GameHandler
{
    state: Option<Rc<RefCell<State>>>,
    last_frame_time: Instant,
    game: T,
    game_manager: Option<GameManager>,
    /*Script*/
    script_editting: Option<ScriptEditting>,
}


impl<T> App<T>
    where T: GameHandler
{
    pub fn new(game: T) -> Self{
        Self { state: None,
            game_manager: None,
            last_frame_time: Instant::now(),
            game,
            script_editting: None
         }
    }

    fn get_dt(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        delta_time.as_secs_f32()
    }
}

    
impl<T> ApplicationHandler for App<T>
    where T: GameHandler
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(PhysicalSize{width: 1920, height: 1080}))
                .unwrap(),
        );

        let state = Rc::new(RefCell::new(pollster::block_on(State::new(window.clone()))));
        
        self.game_manager = Some(GameManager::new(state.clone()));
        let gm = self.game_manager.as_mut().unwrap();
        self.game.on_start(gm);
        
        self.state = Some(state);
        self.last_frame_time = Instant::now();

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let dt = self.get_dt();

        let gm = self.game_manager.as_mut().unwrap();
        self.game.update(gm, dt);

        let state = self.state.as_mut().unwrap();
        let mut state = state.borrow_mut();
        state.input(&event);
        state.update(dt);
        
        for (id, script) in &mut gm.world.query::<&mut components::Script>(){
            script.update(dt, &gm.world, &id);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::MouseInput { device_id:_, state: state_event, button } =>{
                if button == MouseButton::Left && state_event == ElementState::Pressed{
                    println!("{}", state.pick());
                }
            }
            WindowEvent::RedrawRequested => {
                state.render(|game_mananger: &mut GameManager, renderer| {
                    egui::Window::new("Objects").frame(
        Frame::window(&egui::Style::default()).fill(Color32::from_rgba_premultiplied(0, 0, 0, 100))
    )
                .resizable(true)
                .vscroll(true)
                .default_open(true)
                .show(&renderer.context().clone(), |ui| {
                        ui.label(format!("fps: {:.2}", 1.0/dt));
                        for (id, label) in &mut game_mananger.world.query::<&components::Label>(){
                            ui.collapsing(format!("id: {}, label: {}", label.id, label.label), |ui|{
                                let transform = game_mananger.world.get::<&TransformComponent>(id);
                                match transform{
                                    Ok(transform) => {
                                        let mut transform = transform.lock().unwrap();
                                        ui.collapsing("Transform", |ui|{
                                            ui.horizontal(|ui|{
                                                ui.add(egui::Label::new("x: "));
                                                ui.add(egui::DragValue::new(&mut transform.position.x).speed(0.01));
                                                ui.add(egui::Label::new("y: "));
                                                ui.add(egui::DragValue::new(&mut transform.position.y).speed(0.01));
                                                ui.add(egui::Label::new("rotation: "));
                                                ui.add(egui::DragValue::new(&mut transform.rotation.angle).speed(0.01));
                                            });
                                        });
                                        
                                    },
                                    Err(e) => {}
                                }
                                let sprite = game_mananger.world.get::<&components::Sprite>(id);
                                match sprite{
                                    Ok(sprite) => {
                                        ui.collapsing("Sprite", |ui|{
                                            let texture_id = renderer.register_texture(&sprite.texture.view);
                                            ui.image((texture_id, egui::vec2(100.0, 100.0)));
                                        });
                                    }
                                    _ => {}
                                }

                                let script = game_mananger.world.get::<&components::Script>(id);
                                match script {
                                    Ok(script)=>{
                                        ui.collapsing("Script", |ui|{
                                            if ui.button("Edit").clicked(){
                                                self.script_editting = Some(ScriptEditting { entity: id, script: script.get_script() });
                                            }
                                        });
                                    },
                                    _ => {}
                                }
                            });
                        }
                    });

                    let mut close_clicked = false;

                    match &mut self.script_editting {
                        Some(script_editting)=>{
                        egui::Window::new("Script")
                        .frame(
        Frame::window(&egui::Style::default()).fill(Color32::from_rgba_premultiplied(0, 0, 0, 100))
    )
                        .resizable(true)
                        .vscroll(true)
                        .default_open(true)
                        .show(&renderer.context().clone(), |ui| {
                            CodeEditor::default()
                                .id_source("code editor")
                                .with_rows(12)
                                .with_fontsize(11.0)
                                .with_theme(ColorTheme::AYU_DARK)
                                .with_syntax(Syntax::lua())
                                .with_numlines(true)
                                .show(ui, &mut script_editting.script);
                            let mut script = game_mananger.world.get::<&mut components::Script>(script_editting.entity).unwrap();

                            if ui.button("Save").clicked(){
                                script.set_script(script_editting.script.clone());
                            }
                            close_clicked = ui.button("Close").clicked();
                            
                            match &script.state{
                                components::ScriptState::Ok => {},
                                components::ScriptState::Err(e) =>{
                                    ui.label(e);
                                }
                            }

                            
                        });
                        },
                        _ => {}
                    }

                    if close_clicked{
                        self.script_editting = None;
                    }


                    self.game.on_ui(game_mananger, renderer);
                }, gm);
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => (),
        }
    }
}
