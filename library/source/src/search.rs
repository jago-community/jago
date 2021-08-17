author::error!(Incomplete, std::io::Error);

use std::path::PathBuf;

#[test]
fn test_search() {
    let input = "test_search";
    let got = search(dirs::home_dir().unwrap(), input).unwrap();
    let want = dirs::home_dir()
        .unwrap()
        .join("local")
        .join("jago")
        .join("library")
        .join("source")
        .join("src")
        .join("lib.rs");

    assert_eq!(got.as_ref(), vec![want]);
}

pub fn search<'a>(context: PathBuf, input: &'a str) -> Result<Vec<PathBuf>, Error> {
    // let buffers = PathIterator::from(context).filter_map(Result::ok);

    unimplemented!()
}
