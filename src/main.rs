mod action;
mod address;
mod cache;
mod context;
mod environment;
mod image;
mod source;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    match environment::populate() {
        Err(error) => {
            eprintln!("error populating environment: {}\n\n", error);
        }
        _ => {}
    };

    let context = match context::parse(&mut std::env::args()) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("error handling input {}", error);
            std::process::exit(1);
        }
    };

    let code = if let Err(error) = context::handle(context).await {
        eprintln!("error handling action {}", error);
        1
    } else {
        0
    };

    println!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
