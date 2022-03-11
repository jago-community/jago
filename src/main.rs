use context::Context;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn main() {
    let start = std::time::Instant::now();

    let mut code = 0;

    if let Err(error) = logs::before() {
        eprintln!("{:?}", error);

        #[cfg(not(target_arch = "wasm32"))]
        std::process::exit(1);
    }

    log::info!("Starting execution 🧨");

    let context = Context::from("Hello, stranger.");

    #[cfg(feature = "serve")]
    if let Err(error) = http::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(all(not(feature = "serve"), not(target_arch = "wasm32")))]
    if let Err(error) = ansi::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(target_arch = "wasm32")]
    if let Err(error) = browser::watch(context) {
        eprintln!("{:?}", error);
    }

    log::info!("{:?} elapsed", start.elapsed());

    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(code);
}
