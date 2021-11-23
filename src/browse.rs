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
    #[error("Workspace {0}")]
    Workspace(#[from] workspace::Error),
    #[error("NoProfile")]
    NoProfile,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Walk {0}")]
    Walk(#[from] ignore::Error),
    #[error("Walker {0}")]
    Walker(#[from] walkdir::Error),
    #[error("Compress {0}")]
    Compress(#[from] zip::result::ZipError),
    #[error("StripPrefix {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
}

use std::{io::Write, path::PathBuf, process::Command};

use ignore::WalkBuilder;

fn browse(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let firefox = PathBuf::from("/Applications/Firefox.app/Contents/MacOS/firefox");

    let profiles = dirs::data_dir().map_or(Err(Error::NoHome), |path| {
        Ok(path.join("Firefox").join("Profiles"))
    })?;

    let profile_name = "jago-browse";

    let walk = WalkBuilder::new(&profiles)
        .filter_entry(move |entry| match entry.file_type() {
            Some(file_type) if file_type.is_dir() => match entry.file_name().to_str() {
                Some(file_name) if file_name.ends_with(profile_name) => true,
                _ => false,
            },
            _ => false,
        })
        .build();

    let profile = walk
        .filter_map(Result::ok)
        .filter(|entry| entry.path() != profiles)
        .next();

    let profile = match profile {
        Some(profile) => profile,
        _ => {
            Command::new(&firefox)
                .args(["-CreateProfile", profile_name])
                .output()?;

            let walk = WalkBuilder::new(&profiles)
                .filter_entry(move |entry| match entry.file_type() {
                    Some(file_type) if file_type.is_dir() => match entry.file_name().to_str() {
                        Some(file_name) if file_name.ends_with(profile_name) => true,
                        _ => false,
                    },
                    _ => false,
                })
                .build();

            walk.filter_map(Result::ok)
                .filter(|entry| entry.path() != profiles)
                .next()
                .map_or(Err(Error::NoProfile), Ok)?
        }
    };

    // https://github.com/mozilla/web-ext/blob/master/src/cmd/run.js
    //
    // install extension for profile

    let extensions = profile.path().join("extensions");

    if !extensions.exists() {
        std::fs::create_dir(&extensions)?;
    }

    install_extension(&extensions)?;

    Command::new(&firefox)
        .args([
            "--profile",
            profile.path().display().to_string().as_ref(),
            "-install",
            "-extension",
            extensions
                .join("wasm@jago.community.xpi")
                .display()
                .to_string()
                .as_ref(),
        ])
        .output()?;

    Ok(())
}

use std::{fs::OpenOptions, path::Path};

fn install_extension(target: &Path) -> Result<(), Error> {
    let extension = workspace::source_directory()
        .map_err(Error::from)
        .map(|source| {
            source
                .join("crates")
                .join("wasm")
                .join("target")
                .join("pack")
        })?;

    let mut walker = walkdir::WalkDir::new(&extension)
        .into_iter()
        .filter_map(Result::ok);

    let mut target = OpenOptions::new()
        .write(true)
        .create(true)
        .open(target.join("wasm@jago.community.xpi"))?;

    zip_source(&mut walker, &extension, target)?;

    Ok(())
}

use std::{
    fs::File,
    io::{Read, Seek},
};

use ignore::DirEntry;
use zip::{
    write::{FileOptions, ZipWriter},
    CompressionMethod,
};

fn zip_source<T>(
    entries: &mut dyn Iterator<Item = walkdir::DirEntry>,
    prefix: &Path,
    writer: T,
) -> Result<(), Error>
where
    T: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();

    for entry in entries {
        let path = entry.path();
        let name = path.strip_prefix(prefix)?;

        if path.is_file() {
            zip.start_file(name.display().to_string(), options)?;

            let mut source = File::open(path)?;

            source.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }

    zip.finish()?;

    Ok(())
}
