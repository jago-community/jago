use crate::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "pack" => pack(input, context),
        _ => Err(Error::Incomplete),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("TargetNotSupported {0}")]
    TargetNotSupported(Target),
    #[error("WasmPack: {0}")]
    WasmPack(failure::Error),
    #[error("InputOutput: {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("StripPath {0}")]
    StripPath(#[from] std::path::StripPrefixError),
}

use std::path::PathBuf;

use dirs::home_dir;
use ignore::WalkBuilder;
use wasm_pack::command::{
    build::{BuildOptions, Target},
    run_wasm_pack, Command,
};

fn pack(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &mut Context,
) -> Result<(), Error> {
    let _ = input.next();

    let home = home_dir().map_or(Err(Error::NoHome), Ok)?;
    let target = home.join("local").join("jago").join("crates").join("wasm");

    for mode in [Target::Web, Target::NoModules] {
        pack_target(&target, mode)?;
    }

    Ok(())
}

use std::path::Path;

fn pack_target(target: &Path, mode: Target) -> Result<(), Error> {
    let suffix = match mode {
        Target::Web => "web",
        Target::NoModules => "no_modules",
        _ => return Err(Error::TargetNotSupported(mode)),
    };

    let output = PathBuf::from("target").join("pack");

    let mut build_options = BuildOptions::default();
    build_options.path = Some(target.to_owned());
    build_options.target = mode;
    build_options.out_dir = output.join(suffix).display().to_string();

    run_wasm_pack(Command::Build(build_options)).map_err(Error::WasmPack)?;

    let walk = WalkBuilder::new(&target)
        .filter_entry(|entry| {
            entry
                .path()
                .extension()
                .map_or(true, |extension| extension != "rs")
        })
        .build();

    let footprint = target.join(&output);

    for step in walk {
        let entry = match step {
            Ok(entry) => entry,
            Err(error) => {
                #[cfg(feature = "logs")]
                log::error!("stumbled walking package: {}", error);
                drop(error);
                continue;
            }
        };

        match entry.file_type() {
            Some(file_type) if file_type.is_file() => {
                let path = entry.path().strip_prefix(&target)?;

                let target = footprint
                    .join(path)
                    .into_iter()
                    .map(|segment| {
                        if segment == "src" {
                            std::ffi::OsString::from(suffix)
                        } else {
                            segment.to_owned()
                        }
                    })
                    .collect::<PathBuf>();

                #[cfg(feature = "logs")]
                log::info!("packing {} in {}", path.display(), target.display());

                let mut target_directory = target.clone();
                target_directory.pop();

                if !target_directory.exists() {
                    std::fs::create_dir_all(target_directory)?;
                }

                std::fs::copy(entry.path(), target)?;
            }
            _ => {}
        }
    }

    Ok(())
}
