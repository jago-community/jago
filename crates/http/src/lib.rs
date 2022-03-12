mod pack;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pack {0}")]
    Pack(#[from] pack::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
    #[error("Server {0}")]
    Server(#[from] hyper::Error),
    #[error("Address {0}")]
    Address(#[from] std::net::AddrParseError),
}

use ::{
    axum::{http::StatusCode, response, routing::get, routing::get_service, Router},
    instrument::prelude::*,
    std::{
        path::Path,
        sync::{mpsc::channel, Arc, Mutex},
    },
    tokio::runtime::Runtime,
    tower_http::{services::ServeDir, trace::TraceLayer},
};

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

pub fn watch(_: impl Into<Context>) -> Result<(), Error> {
    let target = pack::browser()?;

    let runtime = Runtime::new()?;

    runtime.block_on(async { serve(&target).await })
}

static DOCUMENT: &'static str = include_str!("../../browser/browser.html");

async fn serve(target: &Path) -> Result<(), Error> {
    info!("target: {}", target.display());

    let router = Router::new()
        .route("/", get(|| async { response::Html(DOCUMENT) }))
        .nest(
            "/target",
            get_service(ServeDir::new(target)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled internal error: {}", error),
                )
            }),
        )
        .layer(TraceLayer::new_for_http());

    let address = "0.0.0.0:3000".parse()?;

    debug!("listening at http://{}", &address);

    // run it with hyper on localhost:3000
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .map_err(Error::from)
}
