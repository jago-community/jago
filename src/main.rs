mod color;
mod document;
mod terminal;

use self::document::Document;

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

    let document = Document::from("Hello, stranger");

    if let Err(error) = terminal::watch(document) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
