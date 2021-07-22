author::error!(Incomplete, std::io::Error);

pub fn before() -> Result<Option<Box<dyn Fn()>>, Error> {
    Ok(None)
}

pub fn handle<I: Iterator<Item = String>>(
    _input: &mut std::iter::Peekable<I>,
) -> Result<(), Error> {
    Err(Error::Incomplete)
}
