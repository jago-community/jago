mod interface;
mod request;

fn main() {
    let mut arguments = std::env::args();
    let _programming = arguments.next();

    let argument = match arguments.next() {
        Some(argument) => argument,
        None => {
            std::process::exit(1);
        }
    };

    match request::parse(&argument) {
        Ok(request) => interface::handle(request),
        Err(error) => {
            eprintln!("error handling request: {}", error);
            std::process::exit(1);
        }
    };
}
