author::error!(
    Incomplete,
    base64::DecodeError,
    serde_json::Error,
    hyper::http::Error,
    std::io::Error,
    NoHome,
    crate::address::Error,
    context::Error,
    crate::cache::Error,
    UnexpectedRootValue(serde_json::Value),
    crate::content::Error,
);

use std::collections::HashMap;

use hyper::{Body, Request, Response};

use crate::address;
use crate::cache;

pub fn request<'a>(input: Request<Body>) -> Result<Response<Body>, Error> {
    let (path, variables) = uri(input.uri())?;

    let home = context::home()?;

    let path = match path {
        Either::Left(path) => path,
        Either::Right(address) => {
            cache::ensure(&address)?;
            address.path(&home)
        }
    };

    let output = crate::content::path(&path, &variables)?;

    Response::builder()
        .body(Body::from(output))
        .map_err(Error::from)
}

#[test]
fn map_uri() {
    let cases = vec![
        (
            "?root=local/jago/jago/studio",
            context::home().unwrap().join("local/jago/jago/studio"),
        ),
        (
            "?root=git@github.com:jago-community/jago.git/jago/studio",
            context::home().unwrap().join("remote/jago/jago/studio"),
        ),
        (
            "/jago/studio?root=git@github.com:jago-community/jago.git",
            context::home().unwrap().join("remote/jago/jago/studio"),
        ),
        ("/", context::home().unwrap().join("local/jago/jago/studio")),
    ];

    for (input, want) in cases {
        let input = Uri::builder().path_and_query(input).build().unwrap();
        let got = match uri(&input).unwrap() {
            (Either::Left(path), _) => path,
            (Either::Right(address), _) => address.path(&context::home().unwrap()),
        };
        assert_eq!(want, got);
    }
}

use std::path::PathBuf;

use address::Address;
use either::Either;
use hyper::Uri;
use serde_json::Value;

pub fn uri<'a>(
    input: &'a Uri,
) -> Result<(Either<PathBuf, Address>, HashMap<&'a str, Value>), Error> {
    let home = context::home()?;

    let variables = query(input.query().unwrap_or(""))?;

    let path = input.path();
    let path: &str = path[1..].into();
    let path = variables
        .get("root")
        .map(|value| match &value {
            Value::String(root) => {
                let mut root = root.clone();
                if !path.is_empty() {
                    root.push(std::path::MAIN_SEPARATOR);
                    root.push_str(path);
                }
                Ok(root)
            }
            _ => Err(Error::UnexpectedRootValue(value.clone())),
        })
        .unwrap_or(Ok(String::from("local/jago/jago/studio")))?;

    let output = match address::parse(&path) {
        Ok(address) => Either::Right(address),
        Err(error) => match error.kind {
            address::ErrorKind::Parse(_) => Either::Left(home.join(path)),
            _ => return Err(Error::from(error)),
        },
    };

    Ok((output, variables))
}

pub fn query<'a>(input: &'a str) -> Result<HashMap<&'a str, Value>, Error> {
    let mut variables = HashMap::new();

    for mut pair in input.split("&").map(|segment| segment.split("=")) {
        match (pair.next(), pair.next()) {
            (Some(key), Some(encoded)) if encoded.starts_with("b64:") => {
                let serialized = base64::decode(&encoded[4..])?;
                let value = serde_json::from_slice(&serialized)?;
                variables.insert(key, value);
            }
            (Some(key), Some(value)) => {
                variables.insert(key, Value::String(value.into()));
            }
            _ => {}
        };
    }

    Ok(variables)
}
