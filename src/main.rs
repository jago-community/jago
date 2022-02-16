mod cargo;
mod context;
mod handle;
mod logs;
mod serialize;
mod view;

pub use context::Context;
pub use handle::{Directive, Directives, Handle};
pub use view::View;

pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use cargo::Cargo;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    let mut context = Cargo::default();

    if let Err(error) = context.watch() {
        eprintln!("{:?}", error);
        code = 1;
    }

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
