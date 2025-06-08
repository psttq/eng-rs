pub mod texture;
pub mod egui_tools;
mod render_data;
mod camera;

use std::{env, sync::Arc};
use egui_winit::EventResponse;
use winit::{
    dpi::PhysicalPosition, event::WindowEvent, window::Window
};
use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use egui_wgpu::{wgpu, ScreenDescriptor};

use egui_tools::EguiRenderer;
use render_data::{Instance, Vertex, RECTANGLE_INDICES, RECTANGLE_VERTICES};

use crate::engine::app::{game::components, GameManager};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelMatrixUniform {
    view_proj: [[f32; 4]; 4],
}

impl ModelMatrixUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ObjectIdUniform {
    id: u32,
}


const NUM_INSTANCES_PER_ROW: u32 = 1;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0);

pub struct State {
    window: Arc<Window>,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer, 
    num_indices: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: camera::CameraController,
    egui_renderer: EguiRenderer,
    scale_factor: f32,
    model_matrix_uniform: ModelMatrixUniform,
    model_matrix_buffer: wgpu::Buffer,
    model_matrix_bind_group: wgpu::BindGroup,
    model_matrix_bind_group_layout: wgpu::BindGroupLayout,
    picking_texture: wgpu::Texture,
    picking_view: wgpu::TextureView,
    picking_buffer: wgpu::Buffer,
    object_id_bind_group: wgpu::BindGroup,
    object_id_bind_group_layout: wgpu::BindGroupLayout,
    object_id_buffer: wgpu::Buffer,
    picking_pipeline: wgpu::RenderPipeline,
    mouse_pos: PhysicalPosition<f64>

}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let device = Arc::new(device);

        let limits = device.limits();
        println!(
            "Max sampled textures per shader stage: {}",
            limits.max_sampled_textures_per_shader_stage
        );

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let egui_renderer = EguiRenderer::new(device.clone(), surface_format, None, 1, &window);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

           
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(RECTANGLE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(RECTANGLE_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = RECTANGLE_INDICES.len() as u32;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shader.wgsl").into()),
        });


        /////////////////////////////////////////
        // Camera
        /////////////////////////////////////////

        let camera = camera::Camera {
            position: cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
            speed: 2.0,
            aspect: size.width as f32 / size.height as f32,
            scale: 1.0
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let camera_controller = camera::CameraController::new();
    
        /////////////////////////////////////////
        /////////////////////////////////////////
        /////////////////////////////////////////


        let model_matrix_uniform = ModelMatrixUniform::new();
        let model_matrix_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model Matrix Buffer"),
                contents: bytemuck::cast_slice(&[model_matrix_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let model_matrix_bind_group_layout = device.create_bind_group_layout(&      wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("model_matrix_bind_group_layout"),
        });

        let model_matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_matrix_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_matrix_buffer.as_entire_binding(),
                }
            ],
            label: Some("model_matrix_bind_group"),
        });

        /////////////////////////////////////////
        /////////////////////////////////////////
        /////////////////////////////////////////

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &model_matrix_bind_group_layout
                ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[
                    Vertex::desc()
                ], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        /////////////////////////////////////////
        /////////////////////////////////////////
        /////////////////////////////////////////

        let object_id = ObjectIdUniform { id: 42 };
        let object_id_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object ID Buffer"),
            contents: bytemuck::cast_slice(&[object_id]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let object_id_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Object ID Bind Group Layout"),
            });

        let object_id_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &object_id_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: object_id_buffer.as_entire_binding(),
            }],
            label: Some("Object ID Bind Group"),
        });
        
        let picking_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Picking Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let picking_view = picking_texture.create_view(&Default::default());

        // Buffer to read back picking
        let bytes_per_pixel = 4u32; // Rgba8Uint: 4 байта
        let aligned_bytes_per_row = 256; // минимальное выравнивание

        let buffer_size = aligned_bytes_per_row * 1; // 1 строка
        let picking_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        let picking_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Picking Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../picking_shader.wgsl").into()),
        });

        let picking_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &model_matrix_bind_group_layout,
                    &object_id_bind_group_layout
                ],
            push_constant_ranges: &[],
        });
        
        let picking_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Picking Pipeline"),
            layout: Some(&picking_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &picking_shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[
                    Vertex::desc()
                ], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &picking_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: wgpu::TextureFormat::Rgba8Uint,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            texture_bind_group_layout,
            egui_renderer,
            scale_factor: 1.0,
            model_matrix_uniform,
            model_matrix_buffer,
            model_matrix_bind_group,
            model_matrix_bind_group_layout,
            picking_texture,
            picking_view,
            picking_buffer,
            object_id_bind_group,
            object_id_bind_group_layout,
            object_id_buffer,
            picking_pipeline,
            mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 }
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.camera.aspect = self.size.width as f32 / self.size.height as f32;
        // reconfigure the surface
        self.configure_surface();
    }

    pub fn render<T>(&mut self, mut egui_render_func: T, gm: &mut GameManager)
    where T: FnMut(&mut GameManager, &mut EguiRenderer) -> ()
    {
        let world = &gm.world;
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {r: 10.0/255.0, g: 10.0/255.0, b: 10.0/255.0, a: 1.0}),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderpass.set_pipeline(&self.render_pipeline); 
        renderpass.set_bind_group(1, &self.camera_bind_group, &[]);
        renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); 
        
        for (_id, (_label, sprite, transform_arc)) in &mut world.query::<(&components::Label, &components::Sprite, &components::TransformComponent)>(){
            let transform = transform_arc.lock().unwrap();
            
            let model_matrix_uniform = ModelMatrixUniform {
                view_proj: transform.to_mat().into(),
            };

            let matrix_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model matrix buffer"),
                contents: bytemuck::cast_slice(&[model_matrix_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let model_matrix_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.model_matrix_bind_group_layout, // тот layout, что ты создавал
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                }],
                label: Some("model matrix bind group"),
            });
            renderpass.set_bind_group(2, &model_matrix_bind_group, &[]);
            renderpass.set_bind_group(0, &sprite.texture.bind_group, &[]);
            renderpass.draw_indexed(0..self.num_indices, 0, 0..1 as _);

        }

        drop(renderpass);
        let size = surface_texture.texture.size();
        let picking_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Picking Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let picking_view = picking_texture.create_view(&Default::default());

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &picking_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        

        renderpass.set_pipeline(&self.picking_pipeline); 
        renderpass.set_bind_group(1, &self.camera_bind_group, &[]);
        renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (_id, (label, sprite, transform_arc)) in &mut world.query::<(&components::Label, &components::Sprite, &components::TransformComponent)>(){
            let transform = transform_arc.lock().unwrap();
            
            let model_matrix_uniform = ModelMatrixUniform {
                view_proj: transform.to_mat().into(),
            };

            let matrix_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model matrix buffer"),
                contents: bytemuck::cast_slice(&[model_matrix_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let model_matrix_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.model_matrix_bind_group_layout, // тот layout, что ты создавал
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                }],
                label: Some("model matrix bind group"),
            });

            let object_id = ObjectIdUniform { id: label.id };
            let object_id_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object ID Buffer"),
                contents: bytemuck::cast_slice(&[object_id]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let object_id_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.object_id_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: object_id_buffer.as_entire_binding(),
                }],
                label: Some("Object ID Bind Group"),
            });

            renderpass.set_bind_group(2, &model_matrix_bind_group, &[]);
            renderpass.set_bind_group(0, &sprite.texture.bind_group, &[]);
            renderpass.set_bind_group(3, &object_id_bind_group, &[]);
            renderpass.draw_indexed(0..self.num_indices, 0, 0..1 as _);

        }

 
        drop(renderpass);

        let x = self.mouse_pos.x as u32;
        let y = self.mouse_pos.y as u32;

        if x < self.size.width && y < self.size.height{


        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &picking_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.picking_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row:  Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

    }


        /////////////////////////////////////
        // EGUI
        /////////////////////////////////////

        {
            self.egui_renderer.begin_frame(&self.window);

            egui_render_func(gm, &mut self.egui_renderer);

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.size.width, self.size.height],
                pixels_per_point: self.window.scale_factor() as f32
                    * self.scale_factor,
            };

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &texture_view,
                screen_descriptor,
            );
        }

        /////////////////////////////////////
        /////////////////////////////////////
        /////////////////////////////////////

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn load_texture(&self, name: &str, path: &str) -> texture::Texture{
        let exe_path = env::current_exe().expect("Failed to get executable path");
        let exe_dir = exe_path.parent().expect("Executable has no parent directory");
        let path = exe_dir.join(path);
        let data = std::fs::read(path).unwrap();
        let texture_bytes = data.as_slice();
        let texture = texture::Texture::from_bytes(&self.device, &self.queue, texture_bytes, &self.texture_bind_group_layout, name).unwrap();
        return texture;
    }

    pub fn pick(&self) -> u8{
        let buffer_slice = self.picking_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let id: u8;
        {
            let data = buffer_slice.get_mapped_range();
            id = data[0];
        }
        self.picking_buffer.unmap();
        id
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool{
        let response = self.egui_renderer
            .handle_input(&self.window, &event);
        if response.consumed{
            return true;
        }
        match event{
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.mouse_pos = *position;
            },
            _ => {}
        };
        self.camera_controller.process_events(event);
        false
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

}
