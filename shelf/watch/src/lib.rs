use std::iter::Peekable;

pub fn handle<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut context::Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "watch" => {
            let _ = input.next();
            watch()
        }
        _ => Ok(()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("WindowBuilding {0}")]
    WindowBuilding(#[from] winit::error::OsError),
    #[error("Frame {0}")]
    Frame(#[from] frame::Error),
}

mod font;
mod frame;
mod life;

use frame::Frame;

fn watch() -> Result<(), Error> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)?;

    let mut frame = futures::executor::block_on(Frame::new(window)).map_err(Error::from)?;

    let mut life = life::Universe::new();

    frame.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        frame.spin(&mut life, event, control_flow);
    });
}
