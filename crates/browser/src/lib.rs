#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Logs {0}")]
    Logs(#[from] logs::Error),
    #[error("Window {0}")]
    BuildWindow(#[from] winit::error::OsError),
}

use ::{
    std::sync::{Arc, Mutex},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
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

pub fn watch(context: Context) -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)?;

    #[cfg(target_arch = "wasm32")]
    let log_list = wasm::create_log_list(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[cfg(target_arch = "wasm32")]
        wasm::log_event(&log_list, &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

/*
pub fn watch(context: Context) -> Result<(), Error> {
    log::info!("launching 🧨 {}", context);

    dioxus::web::launch_with_props(app, context.into(), |config| config.rootname("context"));

    Ok(())
}

use dioxus::prelude::*;

fn app(scope: Scope<Context>) -> Element {
    let context = scope.props;

    scope.render(rsx! {
        div { "{context}" }
    })
}
*/
