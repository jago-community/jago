mod color;
mod context;
mod pane;
mod plane;
mod resource;
mod sequence;
mod traits;

use self::{context::Context, plane::Plane, resource::Resource, traits::Outcome};

fn main() {
    let context = Context;

    match context.watch(&context) {
        Ok(Outcome::Exit(code)) => {
            std::process::exit(code.unwrap_or(0));
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
        _ => {}
    };

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

    if let Err(error) = context.watch(plane) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
