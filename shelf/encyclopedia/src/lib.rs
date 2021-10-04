book::error!(Incomplete, std::io::Error, ExpectedField);

use puzzle::Puzzle;
use serde::Serialize;

pub fn handle(context: &Puzzle) -> Result<impl Serialize, Error> {
    Ok(format!("{:?}", context))
}
