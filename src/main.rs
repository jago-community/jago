use context::Context;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(error) = ansi::before() {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(target_arch = "wasm32")]
    if let Err(error) = web::before() {
        eprintln!("{:?}", error);
    }

    let context = Context::from("Hello, stranger.");

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(error) = ansi::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    #[cfg(target_arch = "wasm32")]
    if let Err(error) = web::watch(context) {
        eprintln!("{:?}", error);
    }

    log::info!("{:?} elapsed", start.elapsed());

    #[cfg(not(target_arch = "wasm32"))]
    std::process::exit(code);
}
