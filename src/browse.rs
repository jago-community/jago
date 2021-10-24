use crate::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "browse" => browse(input, context),
        _ => Ok(()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoCache")]
    NoCache,
    #[error("NoData")]
    NoData,
    #[error("NoHome")]
    NoHome,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Walk {0}")]
    Walk(#[from] ignore::Error),
}

use std::{io::Write, path::PathBuf, process::Command};

use ignore::WalkBuilder;

fn browse(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let profiles = dirs::data_dir().map_or(Err(Error::NoHome), |path| {
        Ok(path.join("Firefox").join("Profiles"))
    })?;

    let profile_name = "jago-browse";

    let walk = WalkBuilder::new(dbg!(&profiles))
        .filter_entry(move |entry| match entry.file_type() {
            Some(file_type) if file_type.is_dir() => match entry.file_name().to_str() {
                Some(file_name) if file_name.ends_with(profile_name) => true,
                _ => false,
            },
            _ => false,
        })
        .build();

    let profiles = walk
        .filter_map(Result::ok)
        .filter(|entry| entry.path() != profiles)
        .collect::<Vec<_>>();

    if profiles.len() == 0 {
        let firefox = PathBuf::from("/Applications/Firefox.app/Contents/MacOS/firefox");

        Command::new(firefox)
            .args(["-CreateProfile", profile_name])
            .output()?;
    }

    // https://github.com/mozilla/web-ext/blob/master/src/cmd/run.js
    //
    // install extension for profile

    Ok(())
}
