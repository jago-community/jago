mod ansi;
mod context;
mod handle;
mod logs;

pub mod environment;

pub use context::Context;
pub use handle::{Directive, Directives, Handle};

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    if let Err(error) = logs::before() {
        eprintln!("{:?}", error);
        code = 1;
    }

    let context = Context::get("Hello, stranger.");

    if let Err(error) = ansi::watch(&context) {
        eprintln!("{:?}", error);
        code = 1;
    }

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
