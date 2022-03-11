#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Logs {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
    #[error("WasmPack {0}")]
    WasmPack(Box<dyn std::error::Error + 'static>),
}

use ::{
    std::path::PathBuf,
    wasm_pack::command::{
        build::{BuildOptions, Target},
        run_wasm_pack, Command,
    },
};

pub fn browser() -> Result<PathBuf, Error> {
    let target = environment::target("http", true)?;

    let options = BuildOptions {
        out_dir: format!("{}", target.display()),
        target: Target::Web,
        ..Default::default()
    };

    run_wasm_pack(Command::Build(options))
        .map_err(|error| Error::WasmPack(Box::new(error.compat())))?;

    Ok(target)
}
