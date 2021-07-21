author::error!(Incomplete, base64::DecodeError, serde_json::Error);

use hyper::{Body, Request, Response};

pub fn request(input: Request<Body>) -> Result<Response<Body>, Error> {
    let (path, variables) = uri(input.uri())?;

    unimplemented!()
}

#[test]
fn map_uri() {
    let cases = vec![(
        "?path=jago/studio",
        dirs::home_dir().unwrap().join("local/jago/jago/studio"),
    )];

    for (input, want) in cases {
        let input = Uri::from_static(input);
        let (got, _) = uri(&input).unwrap();
        assert_eq!(want, got);
    }
}

use std::path::PathBuf;

use hyper::Uri;

pub fn uri<'a>(input: &'a Uri) -> Result<(PathBuf, HashMap<&'a str, Value>), Error> {
    let variables = query(input.query().unwrap_or(""))?;

    let path = input.path();
    let mut path = path[1..].into();

    if path == "" {
        if let Some(root) = variables.get("default_path") {
            path = root.to_string();
        }
    }

    Ok((path.into(), variables))
}

use std::collections::HashMap;

use serde_json::Value;

pub fn query<'a>(query: &'a str) -> Result<HashMap<&'a str, Value>, Error> {
    let mut variables = HashMap::new();

    for mut pair in query.split("&").map(|segment| segment.split("=")) {
        match (pair.next(), pair.next()) {
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
