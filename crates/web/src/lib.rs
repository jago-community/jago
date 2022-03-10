#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Logs {0}")]
    Logs(#[from] logs::Error),
}

pub fn before() -> Result<(), Error> {
    logs::before().map_err(Error::from)
}

pub fn watch() -> Result<(), Error> {
    dioxus::web::launch(app);

    Ok(())
}

use dioxus::prelude::*;

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "Hello, stranger." }
    })
}
