mod font;

use std::{iter::Peekable, sync::Arc};

use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

author::error!(
    Incomplete,
    NoFonts,
    CopyFont,
    NoAdaptor,
    ArcGetMut,
    GlyphDraw(String),
    font::Error,
    winit::error::OsError,
    font_kit::error::SelectionError,
    font_kit::error::FontLoadingError,
    wgpu_glyph::ab_glyph::InvalidFont,
    wgpu::RequestDeviceError,
    wgpu::SwapChainError,
    futures::task::SpawnError,
);

pub fn handle<I: Iterator<Item = String>>(_input: &mut Peekable<I>) -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_resizable(true)
        .build(&event_loop)?;

    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let surface = unsafe { instance.create_surface(&window) };

    // Initialize GPU
    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .map(Ok)
            .unwrap_or(Err(Error::NoAdaptor))?;

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(Error::from)
    })?;

    // Create staging belt and a local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(1024);

    let mut local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    let mut view_context = ViewContext::new(window.inner_size())?;
    let mut device_state = DeviceState::default();

    // Prepare swap chain
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: render_format,
            width: view_context.size.width,
            height: view_context.size.height,
            present_mode: wgpu::PresentMode::Fifo,
        },
    );

    let mut glyph_brush = build_brush(&device, render_format, &view_context)?;

    // Render loop
    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        match device_state.handle(&event) {
            Some(Outcome::Change(change)) => {
                let previous_view_context = view_context.clone();

                if let Err(error) = view_context.handle(dbg!(change)) {
                    return log::error!("{}", error);
                }

                println!("handled");

                if &view_context.font.0 != &previous_view_context.font.0 {
                    glyph_brush = match build_brush(&device, render_format, &view_context) {
                        Ok(brush) => brush,
                        Err(error) => return log::error!("{}", error),
                    };
                }

                if &view_context.size != &previous_view_context.size {
                    swap_chain = device.create_swap_chain(
                        &surface,
                        &wgpu::SwapChainDescriptor {
                            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                            format: render_format,
                            width: view_context.size.width,
                            height: view_context.size.height,
                            present_mode: wgpu::PresentMode::Mailbox,
                        },
                    );
                }

                window.request_redraw();
            }
            Some(Outcome::ControlFlow(ControlFlow::Exit)) => std::process::exit(0),
            Some(Outcome::ControlFlow(next)) => *control_flow = next,
            Some(Outcome::Redraw) => {
                // Get a command encoder for the current frame
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Redraw"),
                });

                // Get the next frame
                let maybe_frame = swap_chain.get_current_frame().map_err(Error::from);

                let frame = match maybe_frame {
                    Ok(frame) => frame.output,
                    Err(error) => {
                        return log::error!("{}", error);
                    }
                };

                // Clear frame
                {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear({
                                    let (r, g, b) = view_context.background.into_components();

                                    wgpu::Color {
                                        r: r as f64,
                                        g: g as f64,
                                        b: b as f64,
                                        a: 1.,
                                    }
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                if let Err(error) = draw_text(
                    &window,
                    &device,
                    &mut staging_belt,
                    &frame,
                    &mut encoder,
                    &mut glyph_brush,
                    &view_context,
                ) {
                    return log::error!("{}", error);
                }

                // Submit the work!
                staging_belt.finish();

                queue.submit(Some(encoder.finish()));

                // Recall unused staging buffers
                use futures::task::SpawnExt;

                let spawned = local_spawner
                    .spawn(staging_belt.recall())
                    .map_err(Error::from);

                if let Err(error) = spawned {
                    return log::error!("{}", error);
                }

                local_pool.run_until_stalled();
            }
            _ => {}
        };
    });
}

fn build_brush(
    device: &wgpu::Device,
    render_format: wgpu::TextureFormat,
    view: &ViewContext,
) -> Result<GlyphBrush<()>, Error> {
    let font = ab_glyph::FontArc::try_from_vec(view.font.1.as_ref().clone())?;
    Ok(GlyphBrushBuilder::using_font(font).build(&device, render_format))
}

fn draw_text<'a>(
    window: &'a winit::window::Window,
    device: &'a wgpu::Device,
    staging_belt: &mut wgpu::util::StagingBelt,
    frame: &'a wgpu::SwapChainTexture,
    encoder: &mut wgpu::CommandEncoder,
    brush: &mut wgpu_glyph::GlyphBrush<(), ab_glyph::FontArc>,
    view: &'a ViewContext,
) -> Result<(), Error> {
    let size = window.inner_size();

    let color = color_as_components(view.foreground);

    brush.queue(Section {
        screen_position: (30.0, 30.0),
        bounds: (size.width as f32, size.height as f32),
        text: vec![Text::new("Hello wgpu_glyph!")
            .with_color(color)
            .with_scale(40.0)],
        ..Section::default()
    });

    brush.queue(Section {
        screen_position: (30.0, 90.0),
        bounds: (size.width as f32, size.height as f32),
        text: vec![Text::new("Hello wgpu_glyph!")
            .with_color(color)
            .with_scale(40.0)],
        ..Section::default()
    });

    brush
        .draw_queued(
            device,
            staging_belt,
            encoder,
            &frame.view,
            size.width,
            size.height,
        )
        .map_err(|msg| Error::GlyphDraw(msg))
}

use std::borrow::Cow;

fn draw_triangle<'a>(device: &'a wgpu::Device) -> Result<(), Error> {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            "[[stage(vertex)]]
            fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
                let x = f32(i32(in_vertex_index) - 1);
                let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
                return vec4<f32>(x, y, 0.0, 1.0);
            }

            [[stage(fragment)]]
            fn fs_main() -> [[location(0)]] vec4<f32> {
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }",
        )),
        flags: {
            let mut flags = wgpu::ShaderFlags::empty();

            flags.insert(
                wgpu::ShaderFlags::VALIDATION
            );

            flags
        },
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    unimplemented!()
}

#[derive(Debug)]
enum Outcome {
    Change(ViewChange),
    ControlFlow(ControlFlow),
    Redraw,
}

#[derive(Default)]
struct DeviceState {
    modifiers: Box<ModifiersState>,
}

impl DeviceState {
    fn handle(&mut self, event: &Event<()>) -> Option<Outcome> {
        match event {
            Event::RedrawRequested { .. } => Some(Outcome::Redraw),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => Some(Outcome::ControlFlow(ControlFlow::Exit)),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => Some(Outcome::Change(ViewChange::Resize(*size))),
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                self.modifiers = Box::new(*state);

                None
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode,
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => match (self.modifiers.ctrl(), state, virtual_keycode) {
                (true, ElementState::Released, Some(VirtualKeyCode::C)) => {
                    Some(Outcome::ControlFlow(ControlFlow::Exit))
                }
                (_, ElementState::Released, Some(VirtualKeyCode::R)) => {
                    Some(Outcome::Change(ViewChange::Randomize))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
enum ViewChange {
    Randomize,
    Resize(PhysicalSize<u32>),
}

#[derive(Clone)]
struct ViewContext {
    foreground: palette::Srgb,
    background: palette::Srgb,
    font: (String, Arc<Vec<u8>>),
    size: PhysicalSize<u32>,
}

impl ViewContext {
    fn new(size: PhysicalSize<u32>) -> Result<Self, Error> {
        Ok(Self {
            foreground: palette::Srgb::new(1., 1., 1.),
            background: palette::Srgb::new(0., 0., 0.),
            font: random_font()?,
            size,
        })
    }

    fn handle(&mut self, change: ViewChange) -> Result<(), Error> {
        Ok(match dbg!(change) {
            ViewChange::Randomize => {
                self.random_font()?;
            }
            ViewChange::Resize(size) => {
                self.size = size;
            }
        })
    }

    fn random_font(&mut self) -> Result<(), Error> {
        self.font = random_font()?;
        Ok(())
    }
}

fn color_as_components(color: palette::Srgb) -> [f32; 4] {
    let (r, g, b) = color.into_components();
    [r, g, b, 1.0]
}

fn random_font() -> Result<(String, Arc<Vec<u8>>), Error> {
    println!("1");

    let handle =
        font_kit::sources::fs::FsSource::new().select_by_postscript_name("PT Regular Mono")?;
    //println!("2");
    //let font = list
    //.choose(&mut rand::thread_rng())
    //.map(Ok)
    //.unwrap_or(Err(Error::NoFonts))?;

    //println!("3");
    //let index = match font {
    //font_kit::handle::Handle::Path { font_index, .. } => font_index,
    //font_kit::handle::Handle::Memory { font_index, .. } => font_index,
    //};

    println!("4");

    let font = handle.load()?;

    //println!("5");
    //let key = font
    //.postscript_name()
    //.unwrap_or_else(|| format!("{}/{}/{}", font.family_name(), font.full_name(), index));

    println!("6");

    let output = font
        .copy_font_data()
        .map(Ok)
        .unwrap_or(Err(Error::CopyFont))?;

    println!("7");

    Ok(("PT Mono".into(), output))
}
