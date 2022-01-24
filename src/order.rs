#[test]
fn test_similar() {
    let entries = vec![
        ".git",
        ".gitignore",
        "a",
        "cargo.toml",
        "cargo.lock",
        "entitlements.xml",
        "README.md",
        "jago",
        "jago.vim",
        ".cargo",
        "math",
        ".ds_store",
        "poems",
        "src",
        "target",
    ]
    .into_iter()
    .map(|a| Cow::from(a));

    assert_eq!(similar(entries).get(0), Some(&6));
}

use std::{borrow::Cow, iter::IntoIterator};

pub fn similar<'a>(buffer: &'a str, set: impl IntoIterator<Item = Cow<'a, str>>) -> Vec<usize> {
    set.into_iter()
        .map(|item| score(buffer, &item))
        .map(|_| 0)
        .collect()
}

use num_traits::cast::FromPrimitive;

pub fn score<'a, 'b>(a: &'a str, b: &'b str) -> Option<f32> {
    f32::from_f64(strsim::jaro_winkler(a, b))
}
