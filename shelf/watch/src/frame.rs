use crate::{font, layer::Layer};
use futures::{
    executor::{LocalPool, LocalSpawner},
    task::SpawnExt,
};
use wgpu::{util::StagingBelt, Backends, Device, Instance, Queue, Surface, SurfaceConfiguration};
use wgpu_glyph::{FontId, GlyphBrush, GlyphBrushBuilder, Section, Text};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

pub struct Innards {
    use_color: bool,
    font_id: FontId,
    font_size: f32,
    font_scale: f32,
    font_color: [f32; 4],
    fonts: font::Cache,
}

pub struct Frame {
    window: Window,
    surface_configuration: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    surface: Surface,
    size: PhysicalSize<u32>,
    staging_belt: StagingBelt,
    local_spawner: LocalSpawner,
    local_pool: LocalPool,
    modifiers: ModifiersState,
    font_brush: GlyphBrush<()>,
    layers: Vec<Layer>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("WindowBuilding {0}")]
    WindowBuilding(#[from] winit::error::OsError),
    #[error("FontSelection {0}")]
    FontSelection(#[from] font_kit::error::SelectionError),
    #[error("NoFont")]
    NoFont,
    #[error("FontLoad {0}")]
    FontLoading(#[from] font_kit::error::FontLoadingError),
    #[error("NoFontData")]
    NoFontData,
    #[error("InvalidFont")]
    InvalidFont(#[from] wgpu_glyph::ab_glyph::InvalidFont),
    #[error("NoAdaptor")]
    NoAdaptor,
    #[error("NoRenderFormat")]
    NoRenderFormat,
    #[error("DeviceRequest {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
    #[error("Surface {0}")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("DrawQueue {0}")]
    DrawQueue(String),
    #[error("Spawn {0}")]
    Spawn(#[from] futures::task::SpawnError),
}

impl Frame {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub async fn new(window: Window) -> Result<Self, Error> {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_or(Err(Error::NoAdaptor), Ok)?;

        let (device, queue) = adapter
            .request_device(
                &&wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(Error::from)?;

        let render_format = surface
            .get_preferred_format(&adapter)
            .map_or(Err(Error::NoRenderFormat), Ok)?;

        let size = window.inner_size();

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &surface_configuration);

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let font_brush = GlyphBrushBuilder::using_fonts(vec![]).build(&device, render_format);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipelines = vec![
            crate::pipelines::triangle(&device, &render_pipeline_layout, &surface_configuration),
            crate::pipelines::triangle_editable(
                &device,
                &render_pipeline_layout,
                &surface_configuration,
            ),
        ];

        let layers = vec![
            Layer::Pipeline(
                crate::pipelines::triangle(
                    &device,
                    &render_pipeline_layout,
                    &surface_configuration,
                ),
                Box::new(|frame| frame.use_color),
            ),
            Layer::Pipeline(
                crate::pipelines::triangle_editable(
                    &device,
                    &render_pipeline_layout,
                    &surface_configuration,
                ),
                Box::new(|frame| !frame.use_color),
            ),
            Layer::Handle(Box::new(|frame, view, encoder| {
                let mut frame = frame.lock().unwrap();

                let font_id = match frame.font_id {
                    Some(id) => id,
                    _ => {
                        let font = frame.fonts.pick_font().expect("pick font");

                        let id = frame.font_brush.add_font(font);

                        frame.font_id = Some(id);

                        id
                    }
                };

                frame.font_brush.queue(Section {
                    screen_position: (30.0, 30.0),
                    bounds: (frame.size.width as f32, frame.size.height as f32),
                    text: vec![Text::new("Hello, stranger.")
                        .with_font_id(font_id)
                        .with_color([0.0, 0.0, 0.0, 1.0])
                        .with_scale(40.0)],
                    ..Section::default()
                });

                // Draw the text!
                frame
                    .font_brush
                    .draw_queued(
                        &frame.device,
                        &mut frame.staging_belt,
                        encoder,
                        view,
                        frame.size.width,
                        frame.size.height,
                    )
                    .map_err(Error::DrawQueue)
                    .expect("Draw queued");
            })),
        ];

        Ok(Self {
            window,
            surface_configuration,
            device,
            queue,
            size,
            surface,
            staging_belt,
            local_spawner,
            local_pool,
            modifiers: ModifiersState::empty(),
            fonts: font::Cache::new(),
            font_id: None,
            font_brush,
            layers,
            use_color: true,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_configuration.width = new_size.width;
            self.surface_configuration.height = new_size.height;
            self.surface
                .configure(&self.device, &self.surface_configuration);
        }
    }

    fn get_font_id(&mut self) -> FontId {
        match self.font_id {
            Some(id) => id,
            _ => {
                let font = self.fonts.pick_font().expect("pick font");

                let id = self.font_brush.add_font(font);

                self.font_id = Some(id);

                id
            }
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let pipelines = self.layers.iter().filter_map(|layer| match layer {
            Layer::Pipeline(pipeline, predicate) => {
                if predicate(&self) {
                    Some(pipeline)
                } else {
                    None
                }
            }
            _ => None,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            for pipeline in pipelines {
                render_pass.set_pipeline(pipeline);
            }

            render_pass.draw(0..3, 0..1);
        }

        use std::sync::{Arc, Mutex};

        let state = Arc::new(Mutex::new(*self));

        let handlers = self.layers.iter_mut().filter_map(|layer| match layer {
            Layer::Handle(handle) => Some(handle),
            _ => None,
        });

        for handle in handlers {
            handle(state.clone(), &view, &mut encoder);
        }

        /*
        let font_id = match self.font_id {
            Some(id) => id,
            _ => {
                let font = self.fonts.pick_font().expect("pick font");

                let id = self.font_brush.add_font(font);

                self.font_id = Some(id);

                id
            }
        };

        self.font_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new("Hello, stranger.")
                .with_font_id(font_id)
                .with_color([0.0, 0.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });

        // Draw the text!
        self.font_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                self.size.width,
                self.size.height,
            )
            .map_err(Error::DrawQueue)
            .expect("Draw queued");*/

        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.local_spawner
            .spawn(self.staging_belt.recall())
            .map_err(Error::from)
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        Ok(())
    }

    fn input(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                self.use_color = *state == ElementState::Released;
                true
            }
            winit::event::WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        state: winit::event::ElementState::Released,
                        virtual_keycode,
                        ..
                    },
                ..
            } => match (self.modifiers.ctrl(), virtual_keycode) {
                (true, Some(winit::event::VirtualKeyCode::R)) => {
                    self.font_id = None;
                    true
                }
                (true, Some(winit::event::VirtualKeyCode::C)) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn handle(
        &mut self,
        event: Event<()>,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) -> Result<(), Error> {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                if !self.input(event, control_flow) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            self.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                //self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                self.window.request_redraw();
            }
            _ => {}
        };

        Ok(())
    }

    pub fn spin(
        &mut self,
        life: &mut crate::life::Universe,
        event: Event<()>,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => self.resize(new_size),
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                ..
            } => self.resize(*new_inner_size),
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ModifiersChanged(state),
                ..
            } => {
                self.modifiers = state;
            }
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Released,
                                virtual_keycode,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                match (self.modifiers.ctrl(), virtual_keycode) {
                    (true, Some(winit::event::VirtualKeyCode::R)) => {
                        self.font_id = None;
                        self.window.request_redraw();
                    }
                    (true, Some(winit::event::VirtualKeyCode::C)) => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => {}
                };
            }
            winit::event::Event::RedrawRequested { .. } => {
                // Get a command encoder for the current frame
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        });

                // Get the next frame
                let frame = self
                    .surface
                    .get_current_texture()
                    .map_err(Error::from)
                    .expect("hgjfkhg");
                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Clear frame
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.4,
                                g: 0.4,
                                b: 0.4,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                //render_pass.set_pipeline(&self.render_pipelines[1]); // 2.
                render_pass.draw(0..3, 0..1); // 3.

                drop(render_pass);

                let font_id = match self.font_id {
                    Some(id) => id,
                    _ => {
                        let font = self.fonts.pick_font().expect("pick font");

                        let id = self.font_brush.add_font(font);

                        self.font_id = Some(id);

                        id
                    }
                };

                self.font_brush.queue(Section {
                    screen_position: (30.0, 30.0),
                    bounds: (self.size.width as f32, self.size.height as f32),
                    text: vec![Text::new(&life.render())
                        .with_font_id(font_id)
                        .with_color([0.0, 0.0, 0.0, 1.0])
                        .with_scale(40.0)],
                    ..Section::default()
                });

                // Draw the text!
                self.font_brush
                    .draw_queued(
                        &self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        view,
                        self.size.width,
                        self.size.height,
                    )
                    .map_err(Error::DrawQueue)
                    .expect("Draw queued");

                // Submit the work!

                self.staging_belt.finish();
                self.queue.submit(Some(encoder.finish()));
                frame.present();

                self.local_spawner
                    .spawn(self.staging_belt.recall())
                    .map_err(Error::from)
                    .expect("Recall staging belt");

                self.local_pool.run_until_stalled();
            }
            Event::MainEventsCleared => {
                life.tick();
                self.window.request_redraw();
            }
            _ => {
                *control_flow = winit::event_loop::ControlFlow::Wait;
            }
        }
    }
}
