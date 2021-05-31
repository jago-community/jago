mod action;
mod address;
mod context;
mod document;
mod environment;
mod image;
mod input;
mod interface;
mod parse;
mod source;
mod write;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    match environment::populate() {
        Err(error) => {
            eprintln!("error populating environment: {}\n\n", error);
        }
        _ => {}
    };

    //let mut arguments = std::env::args().skip(1);

    //let argument = match arguments.next() {
    //Some(argument) => argument,
    //None => {
    //std::process::exit(1);
    //}
    //};

    //let input = match input::parse(&argument) {
    //Ok(input) => input,
    //Err(error) => {
    //eprintln!("error handling request: {}", error);
    //std::process::exit(1);
    //}
    //};

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
