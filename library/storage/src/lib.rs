use std::{
    iter::Peekable,
    path::{Path, PathBuf},
};

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "storage" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    match input.next() {
        Some(action) if &action == "link" => link(input)?,
        Some(action) if &action == "watch" => watch(input)?,
        _ => return Err(Error::Incomplete),
    };

    Ok(())
}

pub fn link<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let this = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;
    let to = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;

    create_link(&this, &to)
}

#[cfg(unix)]
pub fn create_link(target: &Path, destination: &Path) -> Result<(), Error> {
    println!("{} -> {}", target.display(), destination.display());
    std::os::unix::fs::symlink(target, destination).map_err(Error::from)
}

#[cfg(windows)]
pub fn create_link(target: &Path, destination: &Path) -> Result<(), Error> {
    let metadata = std::fs::metadata(target)?;

    if metadata.is_file() {
        std::os::windows::fs::symlink_file(target, destination)
    } else {
        std::os::windows::fs::symlink_dir(target, destination)
    }
    .map_err(Error::from)
}

use notify::{
    event::{CreateKind, EventKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use std::sync::mpsc::channel;

pub fn watch<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let root = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;
    let handler = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new_immediate(move |result| {
        match result {
            Ok(event) => match tx.send(event) {
                Err(error) => log::error!("error sending message to receiver {}", error),
                _ => {}
            },
            Err(error) => {
                log::error!("watch error: {}", error);
            }
        };
    })?;

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        watcher.watch(&root, RecursiveMode::NonRecursive)?;

        for event in rx {
            match &event.kind {
                EventKind::Create(CreateKind::Folder) => {
                    for path in &event.paths {
                        let context = path.parent();

                        if Some(root.as_ref()) == context {
                            watcher.watch(&path, RecursiveMode::NonRecursive)?;

                            for entry in std::fs::read_dir(path)? {
                                let entry = entry?;
                                let path = entry.path();

                                execute_handler(&path, &handler)?;
                            }
                        } else {
                            execute_handler(&path, &handler)?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    })
}

fn execute_handler(target: &Path, handler: &Path) -> Result<(), Error> {
    use std::{io::Write, process::Command};

    let output = Command::new(handler).arg(target).output()?;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    handle
        .write_all(output.stdout.as_slice())
        .map_err(Error::from)
}

author::error!(Incomplete, std::io::Error, notify::Error,);
