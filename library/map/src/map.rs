author::error!(
    Incomplete,
    base64::DecodeError,
    serde_json::Error,
    hyper::http::Error,
    std::io::Error,
    NoHome,
    crate::address::Error,
);

use std::{
    fs::File,
    io::{BufReader, Read},
};

use hyper::{Body, Request, Response};

use crate::address;

pub fn request<'a>(input: Request<Body>) -> Result<Response<Body>, Error> {
    let (path, _variables) = uri(input.uri())?;

    let file = File::open(&path)?;
    let mut file = BufReader::new(file);

    let mut output = vec![];

    file.read_to_end(&mut output)?;

    Response::builder()
        .body(Body::from(output))
        .map_err(Error::from)
}

#[test]
fn map_uri() {
    let cases = vec![
        (
            "?root=local/jago/jago/studio",
            dirs::home_dir().unwrap().join("local/jago/jago/studio"),
        ),
        (
            "?root=git@github.com:jago-community/jago.git/jago/studio",
            dirs::home_dir().unwrap().join("remote/jago/jago/studio"),
        ),
        (
            "/jago/studio?root=git@github.com:jago-community/jago.git",
            dirs::home_dir().unwrap().join("remote/jago/jago/studio"),
        ),
    ];

    for (input, want) in cases {
        let input = Uri::builder().path_and_query(input).build().unwrap();
        let (got, _) = uri(&input).unwrap();
        assert_eq!(want, got);
    }
}

use std::path::PathBuf;

use hyper::Uri;
use serde_json::Value;

pub fn uri<'a>(input: &'a Uri) -> Result<(PathBuf, HashMap<&'a str, Value>), Error> {
    let variables = query(input.query().unwrap_or(""))?;

    let home = dirs::home_dir().map_or(Err(Error::NoHome), Ok)?;

    let path = input.path();
    let path: &str = path[1..].into();
    let path = match variables.get("root") {
        Some(Value::String(root)) => {
            let mut root = root.clone();
            root.push(std::path::MAIN_SEPARATOR);
            root.push_str(path);
            root
        }
        _ => path.into(),
    };
    let path = match address::parse(&path) {
        Ok(address) => address.path(&home),
        Err(error) => match error.kind {
            address::ErrorKind::Parse(_) => home.join(path),
            _ => return Err(Error::from(error)),
        },
    };

    Ok((path, variables))
}

use std::collections::HashMap;

pub fn query<'a>(input: &'a str) -> Result<HashMap<&'a str, Value>, Error> {
    let mut variables = HashMap::new();

    for mut pair in input.split("&").map(|segment| segment.split("=")) {
        match (pair.next(), pair.next()) {
            (Some("root"), Some(value)) => {
                variables.insert("root", Value::String(value.into()));
            }
            (Some(key), Some(encoded)) => {
                let serialized = base64::decode(encoded)?;
                let value = serde_json::from_slice(&serialized)?;
                variables.insert(key, value);
            }
            _ => {}
        };
    }

    Ok(variables)
}
