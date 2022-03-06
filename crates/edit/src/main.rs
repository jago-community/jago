use std::{env, path::PathBuf};

fn main() {
    if let Err(error) = editor() {
        eprintln!("error: {}", error);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

fn editor() -> Result<(), Error> {
    let root = env::current_dir()?;

    let directory = read_directory(&root)?;

    let mut buffer = std::io::stdout();

    let (x, y) = size()?;

    let mut serializer = Serializer::new(&mut buffer, (x, y));

    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async move {
        let reader = EventStream::new();

        if self.handle(&Event::Resize(x, y)).stop() {
            return Ok(());
        }

        //serializer.consume(EnterAlternateScreen)?;
        //serializer.consume(Hide)?;
        //serializer.consume(MoveTo(0, 0))?;

        self.serialize(&mut serializer)?;

        serializer.flush()?;

        enable_raw_mode()?;

        let (_, _) = reader
            .flat_map(|result| stream::iter(result.ok()))
            .map(|event| self.handle(&event))
            .map(|directives| -> Result<Directives, Error> {
                serializer.consume(Clear(ClearType::All))?;
                serializer.consume(MoveTo(0, 0))?;

                self.serialize(&mut serializer)?;

                serializer.flush()?;

                Ok(directives)
            })
            .flat_map(|result| stream::iter(result))
            .filter(|directives| future::ready(directives.stop()))
            .into_future()
            .await;

        //disable_raw_mode()?;

        //serializer.consume(Show)?;
        //serializer.consume(LeaveAlternateScreen)?;

        Ok(())
    })
}

use ::{
    crdts::{CmRDT, List},
    std::path::Path,
};

type Actor = u8;

fn read_directory(path: &Path) -> Result<List<PathBuf, Actor>, Error> {
    let directory = std::fs::read_dir(path)?;

    let mut list = List::new();

    let paths = directory
        .into_iter()
        .flat_map(|result| result.ok())
        .map(|entry| entry.path());

    for path in paths {
        let op = list.append(path, 0);

        list.apply(op);
    }

    Ok(list)
}
