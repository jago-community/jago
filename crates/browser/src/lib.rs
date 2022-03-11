#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Logs {0}")]
    Logs(#[from] logs::Error),
    #[error("Window {0}")]
    BuildWindow(#[from] winit::error::OsError),
    #[error("NoWindow")]
    NoWindow,
    #[error("NoDocument")]
    NoDocument,
    #[error("NoBody")]
    NoBody,
    #[error("External {0}")]
    External(String),
}

impl From<wasm_bindgen::JsValue> for Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::External(format!("{:?}", value))
    }
}

use ::{
    std::sync::{Arc, Mutex},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::web::WindowExtWebSys,
        window::{Window, WindowBuilder},
    },
};

use context::{Directive, Directives, Handle};

pub struct Context {
    inner: Arc<Mutex<context::Context>>,
}

impl From<context::Context> for Context {
    fn from(inner: context::Context) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

pub fn watch(context: impl Into<Context>) -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Jago.")
        .build(&event_loop)?;

    let mount = mount_document(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        log::info!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn mount_document(window: &Window) -> Result<web_sys::HtmlCanvasElement, Error> {
    let canvas = window.canvas();

    let window = web_sys::window().ok_or(Error::NoWindow)?;
    let document = window.document().ok_or(Error::NoDocument)?;
    let body = document.body().ok_or(Error::NoBody)?;

    log::info!("um");

    body.style().set_css_text("margin: 0;");

    canvas.style().set_css_text(
        "\
            background-color: #333;\
            max-width: 100%;\
            max-height: 100%;\
        ",
    );

    body.append_child(&canvas)?;

    Ok(canvas)
}
