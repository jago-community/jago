mod context;
mod handle;
mod serialize;
mod view;

pub use context::Context;
pub use handle::{Directives, Handle};
pub use view::View;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    let mut message = String::from("Hello, stranger.");

    if let Err(error) = message.watch() {
        eprintln!("{:?}", error);
        code = 1;
    }

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
