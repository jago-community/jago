use context::Context;

use std::iter::Peekable;

pub fn handle<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _: &'a mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "clean" => {
            drop(input.next());

            let pattern = input.next().map(PathBuf::from).ok_or(Error::Incomplete)?;
            let shift = input.next().map(PathBuf::from).ok_or(Error::Incomplete)?;
            let root = input
                .next()
                .map(PathBuf::from)
                .map_or_else(|| dirs::home_dir().ok_or(Error::NoHome), Ok)?;

            clean_match(root, pattern, shift)?;
        }
        _ => {}
    };

    Ok(())
}

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("NoResourceDirectory")]
    NoResourceDirectory,
    #[error("NoParent")]
    NoParent,
    #[error("SendPath {0}")]
    SendPath(#[from] tokio::sync::mpsc::error::SendError<PathBuf>),
    #[error("Terminal {0}")]
    Terminal(#[from] Box<std::io::Error>),
}

pub fn source_directory() -> Result<PathBuf, Error> {
    dirs::home_dir().map_or(Err(Error::NoHome), |home| Ok(home.join("jago")))
}

pub fn resource_directory() -> Result<PathBuf, Error> {
    let mut current = std::env::current_exe()?;

    loop {
        if !current.pop() {
            return Err(Error::NoResourceDirectory);
        }

        if current.join("Resources").exists() {
            return Ok(current.join("Resources"));
        } else if current.join("assets").exists() {
            return Ok(current);
        }
    }
}

use std::{io::Read, sync::Arc};

use crossterm::{
    cursor::position,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ignore::{WalkBuilder, WalkState};
use tokio::runtime::Runtime;

pub fn clean_match<'a>(root: PathBuf, pattern: PathBuf, shift: PathBuf) -> Result<(), Error> {
    log::info!("searching {}", root.display());
    log::info!("for pattern {}", pattern.display());
    log::info!("and shifting {}", shift.display());

    let jago = source_directory()?;

    let root = Arc::new(root);
    let pattern = Arc::new(pattern);
    let shift = Arc::new(shift);

    let mut builder = WalkBuilder::new(root.as_ref());

    if let Some(error) = builder.add_ignore(jago.join("ignore")) {
        log::error!("error applying ignore file: {}", error);
    }

    let runtime = Runtime::new()?;

    let (match_sender, mut matches) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();

    runtime.spawn(async move {
        if let Err(error) = enable_raw_mode().map_err(Error::from) {
            log::error!("error enabling raw mode: {}", error);
            return;
        }

        while let Some(catch) = matches.recv().await {
            let remove = match read_bool(&format!("remove {:?} (y/n)?", catch)) {
                Ok(answer) => answer,
                Err(error) => {
                    log::error!("error reading answer: {}", error);
                    return;
                }
            };

            if !remove {
                return;
            }

            log::info!("removing {}", catch.display());

            if catch.is_dir() {
                if let Err(error) = std::fs::remove_dir_all(&catch) {
                    log::error!("unable to remove directory {}: {}", catch.display(), error);
                }
            } else {
                if let Err(error) = std::fs::remove_file(&catch) {
                    log::error!("unable to remove file {}: {}", catch.display(), error);
                }
            }
        }

        if let Err(error) = disable_raw_mode().map_err(Error::from) {
            log::error!("error disabling raw mode: {}", error);
            return;
        }
    });

    runtime.block_on(async {
        builder.build_parallel().run(|| {
            Box::new(|maybe| {
                if let Ok(entry) = maybe {
                    let path = entry.path();

                    log::trace!("clean_match checking {:?}", path);

                    if path.ends_with(pattern.as_ref()) {
                        let catch = match path.parent().ok_or(Error::NoParent) {
                            Ok(parent) => parent.join(shift.as_ref()),
                            Err(error) => {
                                log::error!("weird error: {}", error);

                                return WalkState::Continue;
                            }
                        };

                        if catch.exists() {
                            if let Err(error) = match_sender.send(catch).map_err(Error::from) {
                                log::error!("{}", error);
                                return WalkState::Quit;
                            }
                        }
                    }
                }

                WalkState::Continue
            })
        });

        Ok(())
    })
}

fn read_bool(prompt: &str) -> Result<bool, Error> {
    let mut stdout = std::io::stdout();

    loop {
        // execute!(stdout, crossterm::cursor::MoveToColumn(0))?;

        println!("{}", prompt);

        execute!(stdout, crossterm::cursor::MoveUp(1))?;
        // execute!(stdout, crossterm::cursor::MoveToColumn(0))?;

        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                execute!(stdout, crossterm::cursor::MoveToNextLine(1))?;

                return Ok(false);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                execute!(stdout, crossterm::cursor::MoveToNextLine(1))?;

                return Ok(false);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(key),
                ..
            }) if key == 'y' || key == 'n' => {
                execute!(stdout, crossterm::cursor::MoveToNextLine(1))?;

                return Ok(key == 'y');
            }
            _ => {}
        };
    }
}
