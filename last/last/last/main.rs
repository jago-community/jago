mod color;
mod iter_view;
mod order;
mod pane;

use pane::Pane;

fn main() {
    let current = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = Pane::display(&current) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
