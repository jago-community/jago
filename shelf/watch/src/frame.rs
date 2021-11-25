use std::sync::Arc;

use crate::font;
use futures::{
    executor::{LocalPool, LocalSpawner},
    task::SpawnExt,
};
use wgpu::{
    util::StagingBelt, Backends, Device, Instance, Queue, Surface, SurfaceConfiguration,
    TextureFormat,
};
use wgpu_glyph::{FontId, GlyphBrush, GlyphBrushBuilder, Section, Text};
use winit::{
    dpi::PhysicalSize,
    event::{Event, ModifiersState, WindowEvent},
    window::Window,
};

pub struct Frame {
    window: Window,
    surface_configuration: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_format: TextureFormat,
    surface: Surface,
    size: PhysicalSize<u32>,
    staging_belt: StagingBelt,
    local_spawner: LocalSpawner,
    local_pool: LocalPool,
    modifiers: ModifiersState,
    fonts: font::Cache,
    font_id: Option<FontId>,
    font_brush: GlyphBrush<()>,
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

        Ok(Self {
            window,
            surface_configuration,
            device,
            queue,
            size,
            surface,
            render_format,
            staging_belt,
            local_spawner,
            local_pool,
            modifiers: ModifiersState::empty(),
            fonts: font::Cache::new(),
            font_id: None,
            font_brush,
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
                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
