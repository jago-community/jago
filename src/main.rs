mod buffer;
mod color;
mod cursor;
mod display;
mod slice;

fn main() {
    let source = include_bytes!("../poems/chris-abani/the-new-religion");

    if let Err(error) = display::buffer(source) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
