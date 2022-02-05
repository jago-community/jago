mod color;
mod directives;
mod document;
mod terminal;

fn main() {
    let buffer = directives::buffer::Buffer::from("");
    let shell = directives::shell::Shell::new(buffer);

    if let Err(error) = directives::watch(shell) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
