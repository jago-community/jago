author::error!(Incomplete, std::io::Error, hyper::Error, map::Error,);

use std::{convert::Infallible, iter::Peekable, net::SocketAddr};

use futures::future::{Future, FutureExt};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "serve" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    let runtime = tokio::runtime::Runtime::new()?;

    runtime
        .block_on(async {
            let close = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install CTRL+C signal handler");
            };

            serve(close).await
        })
        .map_err(Error::from)
}

pub type Handle<'a> = futures::future::BoxFuture<'a, Result<(), Error>>;

pub fn serve<'a>(close: impl Future<Output = ()> + 'a + Send) -> Handle<'a> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 1342));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    let server = server.with_graceful_shutdown(async { close.await });

    server.map(|result| result.map_err(Error::from)).boxed()
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(map::request(request)
        .unwrap_or_else(|error| Response::new(Body::from(format!("{}", error)))))
}
