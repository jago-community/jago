mod buffer;
mod color;
mod directory;
mod display;
mod event;
mod resource;
mod slice;

fn main() {
    let current = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let selected = match display::directory(&current) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    if let Some(path) = selected {
        if let Err(error) = display::file(&path) {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
