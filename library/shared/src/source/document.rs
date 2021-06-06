use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    sync::Arc,
};

use bytes::Bytes;

pub struct Document {
    buffer: BufWriter<Vec<u8>>,
}

impl Document {
    fn start(&mut self, path: Arc<PathBuf>) -> Result<(), Error> {
        write!(
            self.buffer,
            "<!doctype html>\
        <html>\
            <head>\
                <meta charset=\"utf-8\">\
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                <title>{context}</title>\
            </head>\
            <body>",
            context = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Jago")
        )?;
        // --
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        write!(self.buffer, "</body></html>")?;
        // --
        Ok(())
    }

    fn file(&mut self, path: Arc<PathBuf>) -> Result<(), Error> {
        let file = std::fs::File::open(path.as_ref())?;
        let file = BufReader::new(file);

        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;

        unimplemented!()
    }

    fn output(self) -> Result<Bytes, Error> {
        Ok(Bytes::from(self.buffer.into_inner()?))
    }
}

pub fn write(path: Arc<PathBuf>) -> Result<Bytes, Error> {
    let metadata = std::fs::metadata(path.as_ref())?;

    let mut buffer = vec![];

    let file = std::fs::File::open(path.as_ref())?;

    let mut document = Document {
        buffer: BufWriter::new(buffer),
    };

    write!(
        document.buffer,
        "<!doctype html>\
        <html>\
            <head>\
                <meta charset=\"utf-8\">\
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                <title>{context}</title>\
            </head>\
            <body>",
        context = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Jago")
    )?;

    //if metadata.is_file() {
    //read_file(target, path, metadata)?;
    //} else {
    //read_directory(target, path)?;
    //}

    write!(document.buffer, "</body></html>")?;

    document.output()
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Write(std::io::IntoInnerError<BufWriter<Vec<u8>>>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Write(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>> for Error {
    fn from(error: std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>) -> Self {
        Self::Write(error)
    }
}
