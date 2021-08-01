author::error!(
    Incomplete,
    context::Error,
    std::io::Error,
    NoPipe,
    Process(i32)
);

use std::iter::Peekable;

pub fn handle<I: Iterator<Item = String>>(mut input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "test" || name == "check" || name == "build" => {
            handle_cargo(&mut input)
        }
        _ => Err(Error::Incomplete),
    }
}

use std::path::{Path, PathBuf};

fn handle_cargo<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let command = input.next().map_or(Err(Error::Incomplete), Ok)?;

    let path: Option<PathBuf> = match input.peek() {
        Some(path) if Path::new(path).exists() => {
            let path = PathBuf::from(path.clone());
            let _ = input.next();
            Some(path)
        }
        _ => None,
    };

    let mut test_input = vec!["cargo".to_string(), command.into()];
    test_input.append(&mut input.collect());

    let crate_path = context::crate_path(path)?;

    system_run(&mut test_input.iter().cloned(), &crate_path)
}

pub fn system_run<'a, I: Iterator<Item = String>>(
    input: &mut I,
    path: &'a Path,
) -> Result<(), Error> {
    use std::io::{BufRead, BufReader};

    let program = input.next().map(Ok).unwrap_or(Err(Error::Incomplete))?;

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
            .for_each(|line| log::info!("{}", line));
    });

    let run = running.wait()?;

    output
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            if run.success() {
                log::info!("{}", line);
            } else {
                log::error!("{}", line);
            }
        });

    if run.success() {
        Ok(())
    } else {
        Err(Error::Process(run.code().unwrap_or(42)))
    }
}
