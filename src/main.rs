mod buffer;
mod color;
mod display;
mod unicode;
//mod grid;

fn main() {
    let source = include_bytes!("../poems/chris-abani/the-new-religion");

    if let Err(error) = display::buffer(source) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
