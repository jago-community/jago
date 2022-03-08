#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(#[from] Box<dyn std::error::Error>),
    #[error("Generic {0}")]
    InvalidFont(#[from] ab_glyph::InvalidFont),
    #[error("DrawQueued {0}")]
    DrawQueued(String),
    #[error("Spawn {0}")]
    TaskSpawn(#[from] futures::task::SpawnError),
    #[error("NoAdaptor")]
    NoAdaptor,
    #[error("RequestDevice {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("Surface {0}")]
    Surface(#[from] wgpu::SurfaceError),
}

use ::{
    futures::{
        executor::{block_on, LocalPool, LocalSpawner},
        task::SpawnExt,
    },
    wgpu::{
        util::StagingBelt, Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor,
        Instance, LoadOp, Operations, PowerPreference, PresentMode, Queue,
        RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
        SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
    },
    wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text},
    winit::{
        dpi::{LogicalSize, PhysicalSize},
        event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
};

use crate::Context;

pub struct Screen {
    window: Window,
    size: PhysicalSize<u32>,
    event_loop: EventLoop<()>,
    local_pool: LocalPool,
    local_spawner: LocalSpawner,
    surface: Surface,
    device: Device,
    queue: Queue,
    render_format: TextureFormat,
    staging_belt: StagingBelt,
    glyph_brush: GlyphBrush<()>,
    modifiers: ModifiersState,
}

pub fn watch(context: &'static Context) -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Jago")
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(LogicalSize::new(600.0, 100.0))
        .build(&event_loop)
        .unwrap();

    let instance = Instance::new(Backends::all());

    let surface = unsafe { instance.create_surface(&window) };

    // Initialize GPU
    let (device, queue) = block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_or(Err(Error::NoAdaptor), Ok)?;

        adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await
            .map_err(Error::from)
    })?;

    // Create staging belt and a local pool
    let mut staging_belt = StagingBelt::new(1024);

    let mut local_pool = LocalPool::new();

    let local_spawner = local_pool.spawner();

    // Prepare swap chain
    let render_format = TextureFormat::Bgra8UnormSrgb;

    let mut size = window.inner_size();

    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        },
    );

    // Prepare glyph_brush
    let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/System/Library/Fonts/Supplemental/Andale Mono.ttf"
    ))?;

    let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    window.set_inner_size(LogicalSize::new(800.0, 200.0));

    let mut modifiers = ModifiersState::default();

    // Render loop
    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                use VirtualKeyCode::*;

                match key {
                    C if modifiers.ctrl() => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(m),
                ..
            } => modifiers = m,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                surface.configure(
                    &device,
                    &SurfaceConfiguration {
                        usage: TextureUsages::RENDER_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: PresentMode::Mailbox,
                    },
                );
            }
            Event::RedrawRequested { .. } => {
                // Get a command encoder for the current frame
                let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Redraw"),
                });

                // Get the next frame
                let frame = surface.get_current_texture().map_err(Error::from).unwrap();

                let view = &frame.texture.create_view(&TextureViewDescriptor::default());

                // Clear frame
                {
                    let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: 0.,
                                    g: 0.,
                                    b: 0.,
                                    a: 0.5,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                let buffer = context.to_string();

                glyph_brush.queue(Section {
                    screen_position: (30.0, 30.0),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![Text::new(&buffer)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(40.0)],
                    ..Section::default()
                });

                // Draw the text!
                glyph_brush
                    .draw_queued(
                        &device,
                        &mut staging_belt,
                        &mut encoder,
                        view,
                        size.width,
                        size.height,
                    )
                    .map_err(Error::DrawQueued)
                    .unwrap();

                // Submit the work!
                staging_belt.finish();
                queue.submit(Some(encoder.finish()));
                frame.present();
                // Recall unused staging buffers
                //use futures::task::SpawnExt;

                local_spawner
                    .spawn(staging_belt.recall())
                    .map_err(Error::from)
                    .unwrap();

                local_pool.run_until_stalled();
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    })
}

impl Screen {
    fn new() -> Result<Self, Error> {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Jago")
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size(LogicalSize::new(600.0, 100.0))
            .build(&event_loop)
            .unwrap();

        let instance = Instance::new(Backends::all());

        let surface = unsafe { instance.create_surface(&window) };

        // Initialize GPU
        let (device, queue) = block_on(async {
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .map_or(Err(Error::NoAdaptor), Ok)?;

            adapter
                .request_device(&DeviceDescriptor::default(), None)
                .await
                .map_err(Error::from)
        })?;

        // Create staging belt and a local pool
        let staging_belt = StagingBelt::new(1024);

        let local_pool = LocalPool::new();

        let local_spawner = local_pool.spawner();

        // Prepare swap chain
        let render_format = TextureFormat::Bgra8UnormSrgb;

        let size = window.inner_size();

        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: render_format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Mailbox,
            },
        );

        // Prepare glyph_brush
        let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "/System/Library/Fonts/Supplemental/Andale Mono.ttf"
        ))?;

        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

        window.set_inner_size(LogicalSize::new(800.0, 200.0));

        let modifiers = ModifiersState::default();

        Ok(Self {
            window,
            size,
            event_loop,
            local_pool,
            local_spawner,
            surface,
            device,
            queue,
            render_format,
            staging_belt,
            glyph_brush,
            modifiers,
        })
    }

    fn watch(self, context: &'static Context) {
        self.window.request_redraw();

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    use VirtualKeyCode::*;

                    match key {
                        C if self.modifiers.ctrl() => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::ModifiersChanged(m),
                    ..
                } => self.modifiers = m,
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    self.size = new_size;

                    self.surface.configure(
                        &self.device,
                        &SurfaceConfiguration {
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            format: self.render_format,
                            width: self.size.width,
                            height: self.size.height,
                            present_mode: PresentMode::Mailbox,
                        },
                    );
                }
                Event::RedrawRequested { .. } => {
                    // Get a command encoder for the current frame
                    let mut encoder =
                        self.device
                            .create_command_encoder(&CommandEncoderDescriptor {
                                label: Some("Redraw"),
                            });

                    // Get the next frame
                    let frame = self
                        .surface
                        .get_current_texture()
                        .map_err(Error::from)
                        .unwrap();

                    let view = &frame.texture.create_view(&TextureViewDescriptor::default());

                    // Clear frame
                    {
                        let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: Some("Render pass"),
                            color_attachments: &[RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(Color {
                                        r: 0.,
                                        g: 0.,
                                        b: 0.,
                                        a: 0.5,
                                    }),
                                    store: true,
                                },
                            }],
                            depth_stencil_attachment: None,
                        });
                    }

                    let buffer = context.to_string();

                    self.glyph_brush.queue(Section {
                        screen_position: (30.0, 30.0),
                        bounds: (self.size.width as f32, self.size.height as f32),
                        text: vec![Text::new(&buffer)
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(40.0)],
                        ..Section::default()
                    });

                    // Draw the text!
                    self.glyph_brush
                        .draw_queued(
                            &self.device,
                            &mut self.staging_belt,
                            &mut encoder,
                            view,
                            self.size.width,
                            self.size.height,
                        )
                        .map_err(Error::DrawQueued)
                        .unwrap();

                    // Submit the work!
                    self.staging_belt.finish();
                    self.queue.submit(Some(encoder.finish()));

                    frame.present();
                    // Recall unused staging buffers
                    //use futures::task::SpawnExt;

                    self.local_spawner
                        .spawn(self.staging_belt.recall())
                        .map_err(Error::from)
                        .unwrap();

                    self.local_pool.run_until_stalled();
                }
                _ => *control_flow = ControlFlow::Wait,
            }
        });
    }

    fn redraw(&mut self, context: &Context) {
        // Get a command encoder for the current frame
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Redraw"),
            });

        // Get the next frame
        let frame = self
            .surface
            .get_current_texture()
            .map_err(Error::from)
            .unwrap();

        let view = &frame.texture.create_view(&TextureViewDescriptor::default());

        // Clear frame
        {
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 0.5,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(&context.to_string())
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });

        // Draw the text!
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                view,
                self.size.width,
                self.size.height,
            )
            .map_err(Error::DrawQueued)
            .unwrap();

        // Submit the work!
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // Recall unused staging buffers

        self.local_spawner
            .spawn(self.staging_belt.recall())
            .map_err(Error::from)
            .unwrap();

        self.local_pool.run_until_stalled();
    }
}
