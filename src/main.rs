mod interface;
mod request;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    let mut arguments = std::env::args().skip(1);

    let argument = match arguments.next() {
        Some(argument) => argument,
        None => {
            std::process::exit(1);
        }
    };

    let request = match request::parse(&argument) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("error handling request: {}", error);
            std::process::exit(1);
        }
    };

    let mut code = 0;

    if let Err(error) = interface::handle(request).await {
        eprintln!("error handling request {}", error);
        code = 1;
    }

    println!("{:?} elapsed", start.elapsed());

    std::process::exit(code);
}
