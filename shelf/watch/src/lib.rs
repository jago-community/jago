use std::iter::Peekable;

mod draw;

pub fn handle<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut context::Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "watch" => {
            let _ = input.next();
            //draw::draw().map_err(Error::from)
            watch()
        }
        _ => Ok(()),
    }
}

mod device;
mod font;
//mod frame;
//mod layer;
mod pipelines;
mod shade;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("WindowBuilding {0}")]
    WindowBuilding(#[from] winit::error::OsError),
    //#[error("Frame {0}")]
    //Frame(#[from] frame::Error),
    #[error("Draw {0}")]
    Draw(#[from] draw::Error),
    #[error("Draw {0}")]
    Device(#[from] device::Error),
}

//use frame::Frame;
use device::Device;

mod life;

fn watch() -> Result<(), Error> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)?;

    //let mut frame = futures::executor::block_on(Frame::new(window)).map_err(Error::from)?;
    let mut device = futures::executor::block_on(Device::new(window)).map_err(Error::from)?;

    //let mut life = life::Universe::new();

    device.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = device.handle(event, control_flow) {
            log::error!("frame error: {}", error);
        }

        //frame.spin(&mut life, event, control_flow);
    });
}
