mod color;
mod context;
mod pane;
mod plane;
mod resource;
mod sequence;
mod traits;

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

    let resource = Resource::from(directory.as_path());

    let plane = Plane::with_dimensions(resource, (x, y));

    let end = Plane::with_dimensions("Goodbye, friend.", (x, y));

    let combo = Sequence::from(vec![Sequence::wrap(&context), Sequence::wrap(&plane),Sequence::wrap(&end)]);

    if let Err(error) = context.watch(combo) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
