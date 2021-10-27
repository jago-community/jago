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
    #[error("NoProfile")]
    NoProfile,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Walk {0}")]
    Walk(#[from] ignore::Error),
    #[error("Compress {0}")]
    Compress(#[from] zip::result::ZipError),
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
        .args(["--profile", profile.path().display().to_string().as_ref()])
        .output()?;

    Ok(())
}

use std::{fs::OpenOptions, path::Path};

fn install_extension(target: &Path) -> Result<(), Error> {
    let extension = dirs::home_dir().map_or(Err(Error::NoHome), |home| {
        Ok(home
            .join("local")
            .join("jago")
            .join("crates")
            .join("wasm")
            .join("target")
            .join("pack"))
    })?;

    let mut target = OpenOptions::new()
        .write(true)
        .create(true)
        .open(target.join("wasm@jago.community.xpi"));

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut target));

    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("hello_world.txt", options)?;
    zip.write(b"Hello, World!")?;

    // Apply the changes you've made.
    // Dropping the `ZipWriter` will have the same effect, but may silently fail
    zip.finish()?;

    let mut proxy = OpenOptions::new().create(true).write(true).open(target)?;

    write!(&mut proxy, "{}", extension.display())?;

    Ok(())
}

use ignore::DirEntry;
use std::io::{Read, Seek};
use zip::{write::FileOptions, CompressionMethod};

fn zip_source<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
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
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            #[cfg(feature = "logs")]
            log::info!("zipping file {}", path.display());
            zip.start_file(name.display(), options)?;
            let mut source = File::open(path)?;

            source.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            #[cfg(feature = "logs")]
            log::info!("zipping directory {}", path.display());
            //zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
