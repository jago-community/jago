author::error!(
    Incomplete,
    std::io::Error,
    globset::Error,
    futures::task::SpawnError,
    NotFound,
);

#[test]
#[ignore]
fn get_path() {
    let root = dirs::home_dir().unwrap();
    let input = "*/jago/jago/studio";
    let want = root.join("jago/studio");
    let got = path(&root, input).unwrap();
    assert_eq!(got, want);
}

use std::path::{Path, PathBuf};

use futures::{channel::mpsc, executor::ThreadPool, future, stream::StreamExt, task::SpawnExt};

pub fn path<'a>(root: &'a Path, input: &'a str) -> Result<PathBuf, Error> {
    let input_pattern = globset::Glob::new(input)?.compile_matcher();

    let mut builder = ignore::WalkBuilder::new(root);

    builder.standard_filters(true);

    let context = builder.build_parallel();

    let (path_sender, path_receiver) = mpsc::channel(5000);

    let executor = ThreadPool::new()?;

    let future = async {
        let path_handle = executor.spawn_with_handle(async move {
            let mut options = path_receiver
                .filter_map(|result: Result<ignore::DirEntry, ignore::Error>| {
                    future::ready(result.map(Some).unwrap_or(None))
                })
                .map(|entry| entry.path().to_path_buf())
                .filter(|path| future::ready(input_pattern.is_match(path)));

            match options.next().await {
                Some(path) => Ok(path),
                None => Err(Error::NotFound),
            }
        })?;

        context.run(|| {
            let mut path_sender = path_sender.clone();

            Box::new(move |result| {
                use ignore::WalkState::{Continue, Quit};

                match path_sender.try_send(result) {
                    Ok(_) => Continue,
                    Err(error) => {
                        log::error!("{}", error);

                        Quit
                    }
                }
            })
        });

        path_handle.await
    };

    futures::executor::block_on(future)
}
