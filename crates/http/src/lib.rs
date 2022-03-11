mod pack;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pack {0}")]
    Pack(#[from] pack::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
}

use ::{
    std::sync::{mpsc::channel, Arc, Mutex},
    tokio::runtime::Runtime,
    warp::Filter,
};

pub struct Context {
    inner: Arc<Mutex<context::Context>>,
}

impl From<context::Context> for Context {
    fn from(inner: context::Context) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

pub fn watch(_: impl Into<Context>) -> Result<(), Error> {
    let target = pack::browser()?;

    let runtime = Runtime::new()?;

    let (change_sender, change_receiver) = channel();

    runtime.spawn(async move {
        for _ in change_receiver.iter() {
            log::info!("saw change, recompiling");

            if let Err(error) = pack::browser() {
                log::error!("pack::browser {}", error);
            }
        }
    });

    let browser = environment::component("browser").map(|path| path.join("src"))?;

    runtime.spawn(async move {
        if let Err(error) = pack::watch(&browser, change_sender) {
            log::error!("pack::watch {}", error);
        }
    });

    runtime.block_on(async {
        let root =
            warp::path::end().map(|| warp::reply::html(include_str!("../../browser/browser.html")));

        let bundle = warp::fs::dir(target);

        warp::serve(root.or(bundle)).run(([0, 0, 0, 0], 3333)).await;

        Ok(())
    })
}
