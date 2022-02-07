mod buffer;
mod context;
mod directives;
mod input;

pub use buffer::Buffer;

fn main() {
    let start = std::time::Instant::now();

    let context = match context::get() {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = directives::watch(context, false) {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    log::info!("{:?} elapsed", start.elapsed());
}
