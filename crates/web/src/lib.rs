#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
    #[error("SetLogger {0}")]
    Environment(#[from] environment::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub fn before() -> Result<(), Error> {
    use log::Level;

    console_log::init_with_level(Level::Debug);

    Ok(())
}

static DOCUMENT: &'static str = include_str!("./index.html");

use crate::{Context, Directive, Handle};

pub fn watch(context: &Context) -> Result<(), Error> {
    dioxus::web::launch(app);

    Ok(())
}

use dioxus::prelude::*;

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "Hello, stranger." }
    })
}
