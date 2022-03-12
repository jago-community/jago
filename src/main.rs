use ::{context::Context, instrument::prelude::*};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();

    #[cfg(not(target_arch = "wasm32"))]
    let mut code = 0;

    instrument::before(&[
        #[cfg(feature = "serve")]
        "tower_http",
    ]);

    info!("Starting execution 🧨.");

    let context = Context::from("Hello, stranger.");

    #[cfg(feature = "serve")]
    if let Err(error) = http::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(feature = "ansi")]
    if let Err(error) = ansi::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(target_arch = "wasm32")]
    if let Err(error) = browser::watch(context) {
        eprintln!("{:?}", error);
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing::info!("{:?} elapsed", start.elapsed());

    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(code);
}
