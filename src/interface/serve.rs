use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{http::StatusCode, Body, Request, Response, Server};

use crate::input::Input;

pub async fn handle() -> Result<(), Error> {
    println!(
        "You are now in \"terminal insert\" mode. Below are some examples of what do to:\n\n\

        - Browse `<control-w> N ,g`. This enters \"terminal normal\" mode and then executes a shortcut.\n\
        - Move cursor to pane to the right \"<control-w> l\". Any directional key works. `<control-w>` is not needed in normal mode.\n\
        - Resize window to the left `<control-w> >` or `<control-w> 10 >` for bigger steps.\n\
        - Quit by navigating back to running pane, ensure \"terminal insert\" mode by pressing `i`, then `<control-c>`\n\
        - You can close this pane but if you do not quit the process it will continue in the background. To close the pane, enter normal mode and type `:q`.",
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 1342));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    let server = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    });

    if let Err(error) = server.await {
        eprintln!("error executing server: {}", error)
    }

    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    let maybe_input = if path == "/" {
        Ok(Default::default())
    } else {
        crate::input::parse(&path[..])
    };

    let input = match maybe_input {
        Ok(input) => input,
        Err(error) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("error parsing request: {}", error)))
                .unwrap())
        }
    };

    if let &Input::Serve(_) = &input {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!("already serving")))
            .unwrap());
    }

    Ok(match super::handle_core(&input) {
        Ok(maybe_output) => match maybe_output {
            Some(output) => Response::builder().body(Body::from(output)).unwrap(),
            None => Response::builder()
                .body(Body::from(format!("{:?}", input)))
                .unwrap(),
        },
        Err(error) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("{}", error)))
            .unwrap(),
    })
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Process(tokio::task::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Process(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(err) => Some(err),
            Error::Process(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Machine(error)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Error {
        Error::Process(error)
    }
}
