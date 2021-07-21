mod shell;

use std::iter::Peekable;

author::error!(
    Incomplete,
    ignore::Error,
    std::path::StripPrefixError,
    std::io::Error,
    Weird(String),
    ExternalProcess(i32),
    NoPipe,
    Fallacy,
);

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "shell" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    match input.next() {
        Some(other) => match &other[..] {
            "run" => run(input),
            "expand" => expand(input),
            _ => Err(Error::Incomplete),
        },
        _ => Err(Error::Incomplete),
    }
}

use std::io::{BufRead, BufReader};

pub fn run<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let program = input.next().map(Ok).unwrap_or(Err(Error::Incomplete))?;

    let root = pick_root()?;
    let context = get_context(&root)?;

    let path = context.path();

    let mut running = std::process::Command::new(program)
        .args(input.collect::<Vec<_>>())
        .current_dir(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let output = running
        .stdout
        .take()
        .map(Ok)
        .unwrap_or(Err(Error::NoPipe))?;
    let output = BufReader::new(output);

    std::thread::spawn(move || {
        output
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| log::info!("{}", line));
    });

    let errors = running
        .stderr
        .take()
        .map(Ok)
        .unwrap_or(Err(Error::NoPipe))?;
    let errors = BufReader::new(errors);

    std::thread::spawn(move || {
        errors
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| log::error!("{}", line));
    });

    let run = running.wait()?;

    if run.success() {
        Ok(())
    } else {
        Err(Error::ExternalProcess(run.code().unwrap_or(42)))
    }
}

pub fn expand<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let _context = get_context(match input.next() {
        Some(path) => path.into(),
        None => std::env::current_dir()?,
    })?;

    unimplemented!()
}

use {
    std::path::{Path, PathBuf},
    tempfile::TempDir,
};

fn pick_root() -> Result<PathBuf, Error> {
    let home = dirs::home_dir()
        .map(Ok)
        .unwrap_or(Err(Error::Weird("no home".into())))?;

    let location = std::env::current_dir()?;
    let mut location: &Path = location.as_ref();

    loop {
        if location.join(".git").exists() || location == home {
            break;
        } else if let Some(parent) = location.parent() {
            location = parent;
        } else {
            return Err(Error::Fallacy);
        }
    }

    Ok(location.into())
}

pub fn get_context<P: Into<PathBuf>>(source: P) -> Result<TempDir, Error> {
    let source = source.into();

    let context = tempfile::Builder::new().prefix("shell").tempdir()?;

    let mut builder = ignore::WalkBuilder::new(&source);
    builder.add_ignore(source.join(".dockerignore"));
    builder.add_ignore(source.join(".gitignore"));

    let walker = builder.build();

    for step in walker {
        let step = step?;
        let path = step.path();

        if path == source {
            continue;
        }

        let local = path.strip_prefix(&source)?;

        let target = context.path().join(&local);

        if path.is_file() {
            handle_file(&path, &target)?;
        } else {
            std::fs::create_dir(target)?;
        }
    }

    Ok(context)
}

fn handle_file(path: &Path, target: &Path) -> Result<(), Error> {
    std::fs::copy(path, target).map(|_| ()).map_err(Error::from)
}
