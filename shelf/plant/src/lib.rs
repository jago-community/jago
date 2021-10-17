#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error(transparent)]
    Cargo(anyhow::Error),
}

use std::iter::Peekable;

use cargo::{
    core::{compiler::CompileMode, Workspace},
    ops::{compile, CompileOptions, Packages},
    util::config::Config,
};

pub fn handle<I: Iterator<Item = String>>(_input: &mut Peekable<I>) -> Result<(), Error> {
    let context = dirs::home_dir().map_or(Err(Error::NoHome), Ok)?;

    let config = Config::default().map_err(Error::Cargo)?;

    let workspace = Workspace::new(
        &context.join("local").join("jago").join("Cargo.toml"),
        &config,
    )
    .map_err(Error::Cargo)?;

    let mut compile_options =
        CompileOptions::new(&config, CompileMode::Build).map_err(Error::Cargo)?;

    compile_options.spec =
        Packages::from_flags(false, vec![], vec!["web".into()]).map_err(Error::Cargo)?;

    compile(&workspace, &compile_options).map_err(Error::Cargo)?;

    Ok(())
}
