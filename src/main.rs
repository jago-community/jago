mod buffer;
mod context1;
mod directives;
mod input;
mod screen;
mod view;

pub use buffer::Buffer;

fn main() {
    let start = std::time::Instant::now();

    let context = match context1::get() {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = input::watch(context.clone()) {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    log::info!("{:?} elapsed", start.elapsed());
}
