mod context;
mod source;
mod traits;

use context::Context;

fn main() {
    let start = std::time::Instant::now();

    let context = match Context::get() {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    log::info!("{:?} elapsed", start.elapsed());
}
