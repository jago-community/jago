fn main() {
    let start = std::time::Instant::now();

    let mut input = std::env::args().skip(1).peekable();
    let mut code = 0;
    let mut after: Vec<Box<dyn Fn()>> = vec![];

    #[cfg(feature = "logs")]
    if let Err(error) = pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()
    {
        eprintln!("{}", error);
        code = weight(error);
    }

    #[cfg(feature = "logs")]
    log::trace!("starting execution");

    let mut context = vec![];

    let bounty = gather(&mut input, &mut context).map(move |_| context);

    inspect(bounty);

    #[cfg(feature = "logs")]
    log::info!("{:?} elapsed", start.elapsed());

    after.iter().for_each(|handle| handle());

    std::process::exit(code as i32);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("Pack {0}")]
    Pack(#[from] pack::Error),
    #[error("Serve {0}")]
    Serve(#[from] serve::Error),
    #[error("Browse {0}")]
    Browse(#[from] browse::Error),
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("JavaScriptObjectNotation")]
    JavaScriptObjectNotation(#[from] serde_json::Error),
    #[error("Reason")]
    Reason(#[from] reason::Error),
}

use std::iter::Peekable;

pub type Context = Vec<u8>;

mod browse;
mod pack;
mod reason;
mod serve;

fn gather<'a, Input: Iterator<Item = String>>(
    input: &mut Peekable<Input>,
    context: &'a mut Context,
) -> Result<(), Error> {
    let handles: &[Box<dyn Fn(&mut Peekable<Input>, &mut Context) -> Result<(), Error>>] = &[
        Box::new(|mut input, mut context| {
            reason::handle(&mut input, &mut context).map_err(Error::from)
        }),
        Box::new(|mut input, mut context| {
            pack::handle(&mut input, &mut context).map_err(Error::from)
        }),
        Box::new(|mut input, mut context| {
            serve::handle(&mut input, &mut context).map_err(Error::from)
        }),
        Box::new(|mut input, mut context| {
            browse::handle(&mut input, &mut context).map_err(Error::from)
        }),
        Box::new(|mut input, mut context| pipe(&mut input, &mut context)),
    ];

    for handle in handles {
        handle(input, context)?;
    }

    Ok(())
}

fn inspect<Bounty: AsRef<[u8]>>(input: Result<Bounty, Error>) -> u32 {
    let bounty = match input {
        Ok(bounty) => bounty,
        Err(error) => {
            #[cfg(feature = "logs")]
            log::error!("{}", error);

            return weight(error);
        }
    };

    match report(bounty.as_ref()) {
        Ok(_) => 0,
        Err(error) => {
            #[cfg(feature = "logs")]
            log::error!("{}", error);

            weight(error)
        }
    }
}

fn weight<Input>(_input: Input) -> u32 {
    1
}

use std::io::{stdin, stdout, Read};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

fn report(input: &[u8]) -> Result<(), Error> {
    #[cfg(feature = "logs")]
    if let Ok(input) = std::str::from_utf8(input) {
        log::info!("{}", input);
    } else {
        log::info!("{:?}", input);
    }

    drop(input);

    Ok(())
}

use std::{io::Write, path::PathBuf};

fn pipe(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if PathBuf::from(next).exists() => {}
        _ => return Ok(()),
    };

    let arguments = input.collect::<Vec<_>>();

    let mut input = stdin();

    loop {
        let length = input.read_u32::<NativeEndian>()?;

        let mut buffer = vec![0; length as usize];
        input.read_exact(&mut buffer)?;

        let message: &str = serde_json::from_slice(&buffer)?;

        let output = serde_json::to_vec(&format!(
            "arguments {:?}\nmessage: {:?}\ncontext: {:?}",
            arguments,
            message,
            std::str::from_utf8(context)
        ))?;

        let mut out = stdout();
        out.write_u32::<NativeEndian>(output.len() as u32)?;
        out.write_all(&output)?;
        out.flush()?;
    }
}
