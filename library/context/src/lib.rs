mod environment;

author::error!(
    NoHome,
    environment::Error,
    std::io::Error,
    NotProject,
    NotCrate
);

pub fn before() -> Result<Option<Box<dyn Fn()>>, Error> {
    environment::populate()?;

    Ok(None)
}

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    static ref HOME: Option<PathBuf> = dirs::home_dir();
}

pub fn home() -> Result<PathBuf, Error> {
    HOME.clone().map_or(Err(Error::NoHome), Ok)
}

#[test]
fn test_project() {
    let not_project = tempfile::tempdir().unwrap();

    let tests = vec![
        (not_project.path().into(), Err(Error::NotProject)),
        (
            home().unwrap().join("local").join("jago").join("bounty"),
            Ok(home().unwrap().join("local").join("jago")),
        ),
    ];

    for (input, want) in tests {
        let got = project(Some(input));

        match (got, want) {
            (Ok(got), Ok(want)) => assert_eq!(got, want),
            (Err(got), Err(want)) => assert_eq!(got.to_string(), want.to_string()),
            _ => unreachable!(),
        };
    }
}

pub fn project<'a>(location: Option<PathBuf>) -> Result<PathBuf, Error> {
    let home = dirs::home_dir().map(Ok).unwrap_or(Err(Error::NoHome))?;

    let location = location
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| std::env::current_dir())?;
    let mut location: &Path = location.as_ref();

    loop {
        if location.join(".git").exists() || location == home {
            break;
        } else if let Some(parent) = location.parent() {
            location = parent;
        } else {
            return Err(Error::NotProject);
        }
    }

    Ok(location.into())
}

#[test]
fn test_crate_path() {
    let tests = vec![
        (dirs::home_dir().unwrap(), Err(Error::NotCrate)),
        (
            project(None)
                .unwrap()
                .join("library")
                .join("map")
                .join("src"),
            Ok(project(None).unwrap().join("library").join("map")),
        ),
    ];

    for (input, want) in tests {
        let got = crate_path(Some(&input));

        match (got, want) {
            (Ok(got), Ok(want)) => assert_eq!(got, want),
            (Err(got), Err(want)) => assert_eq!(got.to_string(), want.to_string()),
            _ => unreachable!(),
        };
    }
}

pub fn crate_path<'a, P: Into<PathBuf>>(current: Option<P>) -> Result<PathBuf, Error> {
    let current = current
        .map(Into::into)
        .map(Ok)
        .unwrap_or_else(|| std::env::current_dir())?;
    let mut current: &Path = current.as_ref();

    let mut ancestors = current.ancestors();

    loop {
        if current.join("Cargo.toml").exists() {
            return Ok(current.into());
        } else if let Some(next) = ancestors.next() {
            current = next;
        } else {
            return Err(Error::NotCrate);
        }
    }
}
