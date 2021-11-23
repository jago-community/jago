use crate::Context;

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
}

use std::path::PathBuf;

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

    let project = workspace::source_directory()?;

    pack_binary(&project)?;

    let target = project.join("crates").join("wasm");

    for mode in [Target::Web, Target::NoModules] {
        pack_target(&target, mode)?;
    }

    Ok(())
}

use std::{io::Write, path::Path};

use cargo::{
    core::{compiler::CompileMode, resolver::features::CliFeatures, Workspace},
    ops::{compile, CompileOptions},
    util::config::Config,
};

fn pack_binary(target: &Path) -> Result<(), Error> {
    let config = Config::default().map_err(Error::Cargo)?;

    let workspace = Workspace::new(&target.join("Cargo.toml"), &config).map_err(Error::Cargo)?;

    let suffix = "debug";

    let mut compile_options =
        CompileOptions::new(&config, CompileMode::Build).map_err(Error::Cargo)?;

    compile_options.cli_features =
        CliFeatures::from_command_line(&[], false, false).map_err(Error::Cargo)?;

    compile(&workspace, &compile_options).map_err(Error::Cargo)?;

    let mirror = PathBuf::from(env!("CARGO_HOME"))
        .join("bin")
        .join("cargo-jago");

    std::fs::copy(target.join("target").join(suffix).join("jago"), &mirror)?;

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
            mirror.display()
        )?;
    }

    Ok(())
}

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
