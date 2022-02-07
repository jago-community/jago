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
    .map(|a| Cow::from(a))
    .collect_vec();

    assert_eq!(
        similar("readme", entries.iter().collect_vec()).next(),
        Some(6)
    );
}

use std::{borrow::Cow, cmp::Ordering, iter::IntoIterator};

use itertools::Itertools;

pub fn similar<'a>(
    buffer: &'a str,
    set: impl IntoIterator<Item = &'a Cow<'a, str>>,
) -> impl Iterator<Item = usize> {
    let buffer = buffer.to_lowercase();

    set.into_iter()
        .map(|item| score(&buffer, &item.to_lowercase()).unwrap_or(0.))
        .enumerate()
        .sorted_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(Ordering::Equal))
        .map(|(index, _)| index)
}

use num_traits::cast::FromPrimitive;

pub fn score<'a, 'b>(a: &'a str, b: &'b str) -> Option<f32> {
    f32::from_f64(strsim::jaro_winkler(a, b))
}
