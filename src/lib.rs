#[cfg(feature = "logs")]
mod logs;

//#[cfg_attr(any(target_os = "android", target_os = "ios"), mobile_entry_point)]
use mobile_entry_point::mobile_entry_point;

//#[cfg_attr(any(target_os = "android", target_os = "ios"), mobile_entry_point)]

#[mobile_entry_point]
pub fn handle() {
    let start = std::time::Instant::now();

    let mut input = std::env::args().skip(1).peekable();
    let mut code = 0;
    let mut after: Vec<Box<dyn Fn()>> = vec![];

    #[cfg(feature = "logs")]
    if let Err(error) = logs::before() {
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
    #[cfg(feature = "pack")]
    #[error("Pack {0}")]
    Pack(#[from] pack::Error),
    #[cfg(feature = "serve")]
    #[error("Serve {0}")]
    Serve(#[from] serve::Error),
    #[error("Browse {0}")]
    Browse(#[from] browse::Error),
    #[error("Pipe {0}")]
    Pipe(#[from] pipe::Error),
    #[error("Reason {0}")]
    Reason(#[from] reason::Error),
    //#[error("Watch {0}")]
    //Watch(#[from] watch::Error),
    #[error("Watch {0}")]
    Handle(#[from] handle::Error),
    //#[error("Glass {0}")]
    //Glass(#[from] glass::Error),
    #[error("Glass {0}")]
    Interface(#[from] interface::Error),
}

use context::Context;

use std::iter::Peekable;

mod browse;

// mod handle;

#[cfg(feature = "pack")]
mod pack;

mod pipe;
mod reason;

#[cfg(feature = "serve")]
mod serve;

fn gather<'a, Input: Iterator<Item = String>>(
    input: &mut Peekable<Input>,
    context: &'a mut Context,
) -> Result<(), Error> {
    log::info!("gathering");

    let handles: &[Box<dyn Fn(&mut Peekable<Input>, &mut Context) -> Result<(), Error>>] = &[
        Box::new(|mut input, mut context| {
            reason::handle(&mut input, &mut context).map_err(Error::from)
        }),
        #[cfg(feature = "pack")]
        Box::new(|mut input, mut context| {
            pack::handle(&mut input, &mut context).map_err(Error::from)
        }),
        #[cfg(feature = "serve")]
        Box::new(|mut input, mut context| {
            serve::handle(&mut input, &mut context).map_err(Error::from)
        }),
        Box::new(|mut input, mut context| {
            browse::handle(&mut input, &mut context).map_err(Error::from)
        }),
        //Box::new(|mut input, mut context| {
        //watch::handle(&mut input, &mut context).map_err(Error::from)
        //}),
        //Box::new(|mut input, mut context| {
        //glass::handle(&mut input, &mut context).map_err(Error::from)
        //}),
        Box::new(|mut input, mut context| {
            interface::handle(&mut input, &mut context).map_err(Error::from)
        }),
        #[cfg(not(feature = "handle"))]
        Box::new(|mut input, mut context| {
            pipe::handle(&mut input, &mut context).map_err(Error::from)
        }),
        #[cfg(feature = "handle")]
        Box::new(|mut input, mut context| {
            handle::grasp(&mut input, &mut context).map_err(Error::from)
        }),
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
