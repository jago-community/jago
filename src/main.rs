mod color;
mod context;
mod handle;
mod pane;
mod plane;
mod resource;
mod sequence;
mod traits;
mod view;

use self::{context::Context, plane::Plane, resource::Resource, sequence::Sequence};

fn main() {
    let context = Context::default();

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

    let hello = Plane::with_dimensions("Hello, stranger.", (x, y));
    let directory = Plane::with_dimensions(Resource::from(directory.as_path()), (x, y));
    let goodbye = Plane::with_dimensions("Goodbye, friend.", (x, y));

    let combo = Sequence::from(vec![
        Sequence::wrap(hello),
        Sequence::wrap(directory),
        Sequence::wrap(goodbye),
    ]);

    if let Err(error) = context.watch(combo) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
