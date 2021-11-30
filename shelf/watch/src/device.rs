use std::sync::{Arc, Mutex};

use bitflags::bitflags;
use futures::{
    executor::{LocalPool, LocalSpawner},
    task::SpawnExt,
};
use wgpu::{
    util::StagingBelt, Backends, Instance, PipelineLayout, Queue, Surface, SurfaceConfiguration,
};
use wgpu_glyph::{FontId, GlyphBrush, GlyphBrushBuilder};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::font;

bitflags! {
    #[derive(Default)]
    struct Loading: u32 {
        const FONT = 0b00000001;
    }
}

#[derive(Default)]
struct State {
    modifiers: ModifiersState,
    font_id: Option<FontId>,
    loading: Loading,
}

pub struct Device {
    window: Window,
    surface_configuration: SurfaceConfiguration,
    device: wgpu::Device,
    queue: Queue,
    surface: Surface,
    size: PhysicalSize<u32>,
    staging_belt: StagingBelt,
    render_pipeline_layout: PipelineLayout,
    local_spawner: LocalSpawner,
    local_pool: LocalPool,
    modifiers: ModifiersState,
    font_brush: GlyphBrush<()>,
    fonts: font::Cache,

    state: Arc<Mutex<State>>,
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

impl Device {
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
            render_pipeline_layout,
            font_brush,
            fonts: font::Cache::new(),

            state: Default::default(),
        })
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
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

    fn render(&mut self) -> Result<(), Error> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render_text(&view, &mut encoder, "Hello, stranger.")?;

        // TODO

        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.local_spawner.spawn(self.staging_belt.recall())?;

        self.local_pool.run_until_stalled();

        Ok(())
    }

    fn render_text(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        text: &str,
    ) -> Result<(), Error> {
        use wgpu_glyph::{Section, Text};

        let mut state = self.state.lock().unwrap();

        let font_id = state.font_id.ok_or(Error::NoFont)?;

        let font_id = match state.font_id {
            Some(font_id) => font_id,
            None => {
                if !state.loading.contains(Loading::FONT) {
                    state.loading.insert(Loading::FONT);

                    //let fonts = Arc::new(self.fonts);

                    self.local_spawner.spawn(async {
                        // let settable_state     =    self.state.lock() . unwrap();
                        // let font  =           fonts.clone().pick_font().unwrap();
                        // let font_id        =      self.font_brush.add_font(font);
                        // settable_state.font_id              =      Some(font_id);
                        // let font_id =        self.fonts.load_font(font_id).await;
                        // self.state.lock().unwrap().font_id       = Some(font_id);
                        // self.state.lock().unwrap().loading.remove(Loading::FONT);
                    });
                }

                return Ok(());
            }
        };

        self.font_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(text)
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
                encoder,
                &view,
                self.size.width,
                self.size.height,
            )
            .map_err(Error::DrawQueue)?;

        // ...
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
                // self.use_color = *state == ElementState::Released;
                // true
                false
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
                    // self.font_id = None;
                    // true
                    false
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
        control_flow: &mut ControlFlow,
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
                // self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(Error::Surface(wgpu::SurfaceError::Lost)) => self.resize(self.size),
                    // The system is out of memory, we should probably quit
                    Err(Error::Surface(wgpu::SurfaceError::OutOfMemory)) => {
                        log::error!("{:?}", Error::Surface(wgpu::SurfaceError::OutOfMemory));
                        *control_flow = ControlFlow::Exit
                    }
                    Err(error) => log::error!("{:?}", error),
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
}
