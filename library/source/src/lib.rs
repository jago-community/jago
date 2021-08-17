mod parse;
mod search;
/*

author::error!(NoHome, std::io::Error, NotFound);

use std::path::PathBuf;

#[test]
fn test_search_path() {
    let input = "test_search_path";
    let got = search_path(input).unwrap();
    let want = dirs::home_dir()
        .unwrap()
        .join("local")
        .join("jago")
        .join("library")
        .join("source")
        .join("src")
        .join("lib.rs");

    assert_eq!(got.as_ref(), &want);
}

use rayon::{iter::once, prelude::*};

use std::{path::Path, sync::Arc};

pub fn search_path<'a>(input: &'a str) -> Result<Arc<PathBuf>, Error> {
    let paths = dirs::home_dir()
        .into_par_iter()
        .filter_map(|path_buf| {
            if path_buf.is_file() {
                Some(once(path_buf).filter_map(Some)// .flat_map_iter(||))
            } else {
                Some(once(path_buf)
                .filter_map(|maybe_directory| {
                    maybe_directory.read_dir().map_or_else(
                        |error| {
                            log::error!("maybe_directory {}", Error::from(error));
                            None
                        },
                        Some,
                    )
                })
                .flat_map_iter(|directory| {
                    directory
                        .filter_map(|maybe_entry| {
                            maybe_entry.map_or_else(
                                |error| {
                                    log::error!("maybe_entry {}", Error::from(error));
                                    None
                                },
                                Some,
                            )
                        })
                        .map(|entry| entry.path())
                        .map(PathBuf::from)
                        .map(Arc::new)
                }))
            }
        })
        .flatten();


    paths
        .find_any(|_path| true)
        .map_or(Err(Error::NotFound), Ok)
}

fn directory(input: PathBuf) -> Option<impl ParallelIterator<Item = Arc<PathBuf>>> {
    if input.is_dir() {
        Some(
            once(input)
                .filter_map(|maybe_directory| {
                    maybe_directory.read_dir().map_or_else(
                        |error| {
                            log::error!("maybe_directory {}", Error::from(error));
                            None
                        },
                        Some,
                    )
                })
                .flat_map_iter(|directory| {
                    directory
                        .filter_map(|maybe_entry| {
                            maybe_entry.map_or_else(
                                |error| {
                                    log::error!("maybe_entry {}", Error::from(error));
                                    None
                                },
                                Some,
                            )
                        })
                        .map(|entry| entry.path())
                        .map(PathBuf::from)
                        .map(Arc::new)
                }),
        )
    } else {
        None
    }
}

struct PathWalker {
    source: Arc<PathBuf>,
}

impl ParallelIterator for PathWalker {
    fn drive_undindexed(consumer: ) -> {

    }
}
*/
