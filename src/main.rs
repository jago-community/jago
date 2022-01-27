mod buffer;
mod color;
mod context;
mod directory;
mod display;
mod event;
mod grid;
mod input;
mod iter_view;
mod order;
mod pane;
mod resource;
mod slice;
mod split;

fn main() {
    let current = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let mut directory = directory::Directory::from(current.as_path());

    directory.set_entries();

    if let Err(error) = display::iterator(
        directory
            .get_entry_paths()
            .into_iter()
            .map(|path| path.display()),
    ) {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    //let selected = match display::directory(&current) {
    //Ok(path) => path,
    //Err(error) => {
    //eprintln!("{}", error);
    //std::process::exit(1);
    //}
    //};

    //if let Some(path) = selected {
    //if let Err(error) = display::file(&path) {
    //eprintln!("{}", error);
    //std::process::exit(1);
    //}
    //}
}
