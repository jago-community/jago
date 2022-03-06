mod cargo;
mod colors;
mod context;
mod directory;
mod handle;
mod logs;
mod serialize;
mod view;
mod window;

pub use context::Context;
pub use handle::{Directive, Directives, Handle};
pub use view::View;

pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use directory::Directory;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    match Directory::current() {
        Ok(mut context) => {
            if let Err(error) = context.watch() {
                eprintln!("{:?}", error);
                code = 1;
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
            code = 1;
        }
    };

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
