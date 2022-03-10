#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SetLogger {0}")]
    Logs(#[from] logs::Error),
}

pub fn before() -> Result<(), Error> {
    logs::before().map_err(Error::from)
}
