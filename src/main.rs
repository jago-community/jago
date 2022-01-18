mod buffer;
mod color;
mod cursor;
mod display;
mod slice;

fn main() {
    if let Err(error) = display::buffer() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
