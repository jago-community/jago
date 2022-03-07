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
}

use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

use crate::Context;

pub fn handle(context: &'static Context) -> Result<(), Error> {
    // Open window and create a surface
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    // Initialize GPU
    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_or(Err(Error::NoAdaptor), Ok)?;

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(Error::from)
    })?;

    // Create staging belt and a local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(1024);
    let mut local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    // Prepare swap chain
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    );

    // Prepare glyph_brush
    let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/System/Library/Fonts/Supplemental/Andale Mono.ttf"
    ))?;

    let mut glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    // Render loop
    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Mailbox,
                    },
                );
            }
            winit::event::Event::RedrawRequested { .. } => {
                // Get a command encoder for the current frame
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Redraw"),
                });

                // Get the next frame
                let frame = surface.get_current_texture().expect("Get next frame");
                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Clear frame
                {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                }

                let buffer = context.to_string();

                glyph_brush.queue(Section {
                    screen_position: (30.0, 30.0),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![Text::new(&buffer)
                        .with_color([0.0, 0.0, 0.0, 1.0])
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
                use futures::task::SpawnExt;

                local_spawner
                    .spawn(staging_belt.recall())
                    .map_err(Error::from)
                    .unwrap();

                local_pool.run_until_stalled();
            }
            _ => {
                *control_flow = winit::event_loop::ControlFlow::Wait;
            }
        }
    })
}
