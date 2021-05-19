mod address;
mod environment;
mod input;
mod interface;
mod parse;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    match environment::populate() {
        Err(error) => {
            eprintln!("error populating environment: {}\n\n", error);
        }
        _ => {}
    };

    let mut arguments = std::env::args().skip(1);

    let argument = match arguments.next() {
        Some(argument) => argument,
        None => {
            std::process::exit(1);
        }
    };

    let input = match input::parse(&argument) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("error handling request: {}", error);
            std::process::exit(1);
        }
    };

    let mut code = 0;

    if let Err(error) = interface::handle(&input).await {
        eprintln!("error handling input {}", error);
        code = 1;
    }

    println!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
