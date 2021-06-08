use std::{borrow::Cow, convert::Infallible, iter::Peekable, net::SocketAddr, sync::Arc};

use hyper::{
    http::StatusCode,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<bool, Error> {
    match input.peek() {
        Some(next) if next == "serve" => input.next(),
        _ => return Ok(false),
    };

    println!(
        "You are now in \"terminal insert\" mode. Below are some examples of what do to:\n\n\

        - Browse `<control-w> N ,g`. This enters \"terminal normal\" mode and then executes a shortcut.\n\
        - Move cursor to pane to the right \"<control-w> l\". Any directional key works. `<control-w>` is not needed in normal mode.\n\
        - Resize window to the left `<control-w> >` or `<control-w> 10 >` for bigger steps.\n\
        - Quit by navigating back to running pane, ensure \"terminal insert\" mode by pressing `i`, then `<control-c>`\n\
        - You can close this pane but if you do not quit the process it will continue in the background. To close the pane, enter normal mode and type `:q`.",
    );

    let runtime = tokio::runtime::Runtime::new()?;

    runtime
        .block_on(async {
            let addr = SocketAddr::from(([0, 0, 0, 0], 1342));

            let make_svc =
                make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

            let server = Server::bind(&addr).serve(make_svc);

            let server = server.with_graceful_shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install CTRL+C signal handler");
            });

            server.await
        })
        .map(|_| true)
        .map_err(Error::from)
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    if request.method() != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from(format!(
                "found: {}\n\nsupported: GET",
                request.method()
            )))
            .unwrap());
    }

    let context = dirs::home_dir().unwrap();

    let mut maybe_address: Option<shared::address::Address> = None;

    let path = request.uri().path();

    let input = &path[1..];

    let maybe_path = match shared::address::parse(&input) {
        Ok(address) => {
            let path = address.full(context.join("cache"));
            maybe_address = Some(address);
            path
        }
        Err(error) => match std::env::var("JAGO") {
            Ok(jago) if jago.len() > 0 => {
                let gonna_try_this = Cow::Owned([&jago, input].join("/"));

                match shared::address::parse(&gonna_try_this) {
                    Ok(address) => {
                        let path = address.full(context.join("cache"));
                        maybe_address = Some(address);
                        path
                    }
                    Err(other_error) => {
                        return Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(format!(
                                "Error finding path:\n\n{}\n\nand\n\n{}",
                                error, other_error
                            )))
                            .unwrap())
                    }
                }
            }
            _ => context.join("local/jago").join(input),
        },
    };

    if let Some(address) = maybe_address {
        if let Err(error) = shared::cache::ensure(&address) {
            return Ok(bad_request(error.into()));
        }
    }

    let path = Arc::new(maybe_path);

    let mut body = std::io::BufWriter::new(vec![]);

    if let Err(error) = shared::source::read(&mut body, path.clone()) {
        Ok(bad_request(Error::from(error)))
    } else {
        match body.into_inner() {
            Ok(body) => Ok(Response::builder().body(Body::from(body)).unwrap()),
            Err(error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("{}", error)))
                .unwrap()),
        }
    }
}

fn bad_request(error: Error) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(format!("{}", error)))
        .unwrap()
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Serve(hyper::Error),
    Source(shared::source::Error),
    Cache(shared::cache::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
            Error::Source(error) => write!(f, "{}", error),
            Error::Cache(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Serve(error) => Some(error),
            Error::Source(error) => Some(error),
            Error::Cache(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Self::Serve(error)
    }
}

impl From<shared::source::Error> for Error {
    fn from(error: shared::source::Error) -> Self {
        Self::Source(error)
    }
}

impl From<shared::cache::Error> for Error {
    fn from(error: shared::cache::Error) -> Self {
        Self::Cache(error)
    }
}
