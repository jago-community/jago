mod pack;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pack {0}")]
    Pack(#[from] pack::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    std::sync::{Arc, Mutex},
    tokio::runtime::Runtime,
    warp::{fs::File, http::HeaderValue, Filter, Reply},
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

    runtime.block_on(async {
        let root =
            warp::path::end().map(|| warp::reply::html(include_str!("../../browser/browser.html")));

        let bundle = warp::fs::dir(target).map(|file: File| match file.path().extension() {
            //Some(ext) if ext == "js" => {
            //let mut response = file.into_response();
            //let headers = response.headers_mut();
            //headers.insert("content-type", HeaderValue::from_static("text/javascript"));
            //response
            //}
            //Some(ext) if ext == "wasm" => {
            //let mut response = file.into_response();
            //let headers = response.headers_mut();
            //headers.insert(
            //"X-Content-Type-Options",
            //HeaderValue::from_static("application/wasm"),
            //);
            //response
            //}
            _ => file.into_response(),
        });

        warp::serve(root.or(bundle)).run(([0, 0, 0, 0], 3333)).await;

        Ok(())
    })
}
