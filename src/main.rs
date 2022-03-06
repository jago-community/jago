mod context;
mod document;

pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub use document::Document;

fn main() {
    let start = std::time::Instant::now();
    let mut code = 0;

    let document = include_str!("./main.rs").chars().into();

    if let Err(error) = context::watch(document) {
        eprintln!("{:?}", error);
        code = 1;
    }

    log::info!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
