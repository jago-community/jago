#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

use ::std::sync::{Arc, Mutex};

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

pub fn watch(context: impl Into<Context>) -> Result<(), Error> {
    // ...
}
