fn main() {
    let start = std::time::Instant::now();

    let mut input = std::env::args().peekable();

    let _program = input.next();

    let mut code = 0;
    let mut after: Option<Box<dyn Fn()>> = None;

    match logger::before() {
        Ok(cleanup) => {
            after = Some(cleanup);
        }
        Err(error) => {
            eprintln!("error starting logger: {}", error);
            code = 1;
        }
    };

    log::info!("starting execution");

    match logger::handle(&mut input) {
        Err(error) => {
            eprintln!("error handling log input: {}", error);
            code = 1;
        }
        _ => {}
    };

    #[cfg(feature = "serve")]
    {
        match server::handle(&mut input) {
            Err(error) => {
                eprintln!("error handling server input: {}", error);
                code = 1;
            }
            _ => {}
        };
    }

    log::info!("{:?} elapsed", start.elapsed());

    if let Some(after) = after {
        after();
    }

    std::process::exit(code);
}

#[derive(Debug)]
enum Error {
    Log(logger::Error),
    #[cfg(feature = "serve")]
    Serve(server::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Log(error) => write!(f, "{}", error),
            #[cfg(feature = "serve")]
            Error::Serve(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Log(error) => Some(error),
            #[cfg(feature = "serve")]
            Error::Serve(error) => Some(error),
        }
    }
}

impl From<logger::Error> for Error {
    fn from(error: logger::Error) -> Self {
        Self::Log(error)
    }
}

#[cfg(feature = "serve")]
impl From<server::Error> for Error {
    fn from(error: server::Error) -> Self {
        Self::Serve(error)
    }
}
