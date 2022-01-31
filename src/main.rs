mod color;
mod display;
mod filter;
mod handle;
mod plane;
mod resource;
mod screen;
mod sequence;
mod traits;
mod view;

use self::{plane::Plane, resource::Resource, screen::Screen, sequence::Sequence};

fn main() {
    let directory = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let (x, y) = match crossterm::terminal::size() {
        Ok((x, y)) => (x as usize, y as usize),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    /*
    let hello = Plane::with_dimensions("Hello, stranger.", (x, y));
    let directory = Plane::with_dimensions(Resource::from(directory.as_path()), (x, y));
    let goodbye = Plane::with_dimensions("Goodbye, friend.", (x, y));

    let mut combo = Sequence::from(vec![
        Sequence::wrap(hello),
        Sequence::wrap(directory),
        Sequence::wrap(goodbye),
    ]);

    if let Err(error) = display::watch(&mut combo) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
    */

    let mut message = "Hello, stranger".to_string();

    if let Err(error) = Screen::watch(&mut message) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
