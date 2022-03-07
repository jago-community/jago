mod context;
mod logger;
mod screen;

use context::Context;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    if let Err(error) = logger::before() {
        eprintln!("{:?}", error);
        code = 1;
    }

    let context = Context::get("Hello, stranger.");

    if let Err(error) = screen::watch(context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
