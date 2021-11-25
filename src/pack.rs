use context::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "pack" => pack(input, context),
        _ => Ok(()),
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
    #[error("NoName {0:?}")]
    NoName(std::path::PathBuf),
    #[error("NoDataDirectory")]
    NoDataDirectory,
    #[error(transparent)]
    Cargo(anyhow::Error),
    #[error("Workspace: {0}")]
    Workspace(#[from] workspace::Error),
    #[error("BundleStatus: {0:?}")]
    BundleStatus(Option<i32>),
}

use std::path::PathBuf;

use ignore::WalkBuilder;
use wasm_pack::command::{
    build::{BuildOptions, Target},
    run_wasm_pack, Command,
};

#[derive(PartialEq)]
enum BuildProfile {
    Dev,
    Release,
}

impl std::fmt::Display for BuildProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BuildProfile::Dev => "debug",
                BuildProfile::Release => "release",
            }
        )
    }
}

fn pack(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &mut Context,
) -> Result<(), Error> {
    let _ = input.next();

    let project = workspace::source_directory()?;

    let profile = BuildProfile::Dev;

    pack_bundle(&project, &profile)?;

    let target = project.join("crates").join("wasm");

    for target_mode in [Target::Web, Target::NoModules] {
        pack_target(&target, target_mode, &profile)?;
    }

    Ok(())
}

use std::{io::Write, path::Path};

fn pack_bundle(target: &Path, profile: &BuildProfile) -> Result<(), Error> {
    let mut args = vec!["bundle", "--features", "handle,logs"];

    if profile == &BuildProfile::Release {
        args.push("--release");
    }

    let bundle_result = std::process::Command::new("cargo")
        .current_dir(target)
        .args(args)
        .status()
        .map_err(Error::from)?;

    if !bundle_result.success() {
        return Err(Error::BundleStatus(bundle_result.code()));
    }

    #[cfg(target_os = "macos")]
    let bundle = target
        .join("target")
        .join(profile.to_string())
        .join("bundle")
        .join("osx")
        .join("jago.app");

    #[cfg(target_os = "macos")]
    let applications = dirs::home_dir()
        .map(|path| path.join("Applications"))
        .map_or(Err(Error::NoHome), Ok)?;

    #[cfg(target_os = "macos")]
    let destination = applications.join("Jago.app");

    if destination.exists() {
        std::fs::remove_dir_all(&destination)?;
    }

    std::fs::rename(dbg!(bundle), &destination)?;

    #[cfg(target_os = "macos")]
    let binary = destination.join("Contents").join("MacOS").join("jago");

    let host_manifest = dirs::data_dir().map_or(Err(Error::NoDataDirectory), |data| {
        Ok(data
            .join("Mozilla")
            .join("NativeMessagingHosts")
            .join("jago.json"))
    })?;

    if !host_manifest.exists() {
        let mut file = std::fs::File::create(&host_manifest)?;

        write!(
            file,
            r#"{{
    "name": "jago",
    "description": "Interact with anarchy.",
    "path": "{}",
    "type": "stdio",
    "allowed_extensions": [ "wasm@jago.community" ]
}}
"#,
            binary.display()
        )?;
    }

    Ok(())
}

fn pack_target(target: &Path, mode: Target, profile: &BuildProfile) -> Result<(), Error> {
    let suffix = match mode {
        Target::Web => "web",
        Target::NoModules => "no_modules",
        _ => return Err(Error::TargetNotSupported(mode)),
    };

    let output = PathBuf::from("target")
        .join(profile.to_string())
        .join("pack");

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
