mod context;
mod grid;

use context::Context;

fn main() {
    let start = std::time::Instant::now();

    let mut context = match Context::get() {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = context.watch() {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    log::info!("{:?} elapsed", start.elapsed());
}
