author::error!(
    Incomplete,
    base64::DecodeError,
    serde_json::Error,
    hyper::http::Error,
    std::io::Error
);

use std::{
    fs::File,
    io::{BufReader, Read},
};

use hyper::{Body, Request, Response};

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
    let cases = vec![(
        "?root=local/jago/jago/studio",
        dirs::home_dir().unwrap().join("local/jago/jago/studio"),
    )];

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

    let path = input.path();
    let path: &str = path[1..].into();

    let home = dirs::home_dir().unwrap();

    let mut root = match variables.get("root") {
        Some(Value::String(root)) => home.join(root),
        _ => home,
    };

    if path != "" {
        root = root.join(path);
    }

    Ok((root, variables))
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
