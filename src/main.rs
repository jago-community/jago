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
}

use std::{iter::Peekable, mem::replace};

pub type Context = Vec<u8>;

mod pack;

fn gather<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut Context,
) -> Result<(), Error> {
    for handle in [reason, pack::handle] {
        handle(input, context)?;
    }

    Ok(())
}

fn reason<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut Context,
) -> Result<(), Error> {
    let _difference = replace(context, b"why things are the way they are".to_vec());

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

    report(bounty.as_ref());

    0
}

fn weight<Input>(_input: Input) -> u32 {
    1
}

fn report(input: &[u8]) {
    #[cfg(feature = "logs")]
    if let Ok(input) = std::str::from_utf8(input) {
        log::info!("{}", input);
    } else {
        log::info!("{:?}", input);
    }

    drop(input);
}
