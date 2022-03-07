use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

pub fn handle(
    event: &Event<()>,
    modifiers: &mut ModifiersState,
    control_flow: &mut ControlFlow,
    size: &mut PhysicalSize<u32>,
) {
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
            event: winit::event::WindowEvent::ModifiersChanged(m),
            ..
        } => modifiers = &mut m,
        Event::WindowEvent {
            event: winit::event::WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } => {
            size = &mut new_size;

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
        Event::RedrawRequested { .. } => {
            // Get a command encoder for the current frame
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Redraw"),
            });

            // Get the next frame
            let frame = surface.get_current_texture().map_err(Error::from).unwrap();

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
}
