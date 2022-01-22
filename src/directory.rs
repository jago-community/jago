use std::path::{Path, PathBuf};

pub struct Directory {
    path: PathBuf,
    read: bool,
    entries: Vec<PathBuf>,
    marker: Option<usize>,
    buffer: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Similarity {0}")]
    Similarity(#[from] fst::Error),
    #[error("SimilarityAutomaton {0}")]
    SimilarityAutomaton(#[from] fst::automaton::LevenshteinError),
    #[error("FromUtf8Error {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Incomplete")]
    Incomplete,
}

impl From<&Path> for Directory {
    fn from(path: &Path) -> Self {
        Self {
            path: path.into(),
            read: false,
            entries: vec![],
            marker: None,
            buffer: String::new(),
        }
    }
}

#[test]
fn selected() {
    let entries = vec![
        ".cargo",
        ".ds_store",
        ".git",
        ".gitignore",
        "a",
        "cargo.lock",
        "cargo.toml",
        "entitlements.xml",
        "jago",
        "jago.vim",
        "math",
        "poems",
        "src",
        "target",
        "README.md",
    ];
    let directory = Directory {
        path: PathBuf::new(),
        read: true,
        entries: entries.clone().into_iter().map(PathBuf::from).collect_vec(),
        marker: None,
        buffer: String::from("readme"),
    };

    assert_eq!(directory.selected(&entries), Some(Path::new("README.md")));
}

impl Directory {
    fn get_entries(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|path| path.strip_prefix(&self.path).ok().map(PathBuf::from))
            .map(|path| path.display().to_string().to_lowercase())
            .sorted()
            .collect()
    }

    fn selected_index<A: AsRef<str>>(&self, entries: &[A]) -> Option<usize> {
        let mut index = None;

        if self.buffer.len() > 0 {
            let mut sorted_entry_indices = most_similar_to(
                entries.iter().map(|entry| entry.as_ref()),
                self.buffer.as_ref(),
            )
            .ok()?;

            if let Some(first) = sorted_entry_indices.next() {
                index = Some(first);
            }
        } else if let Some(marker) = self.marker {
            index = Some(marker);
        }

        index
    }

    fn selected(&self, entries: &[&str]) -> Option<&Path> {
        self.selected_index(entries)
            .iter()
            .flat_map(|index| self.entries.get(*index))
            .map(|item| item.as_path())
            .next()
    }
}

use crate::color::ColorPicker;

use itertools::Itertools;

use fst::{automaton::Levenshtein, IntoStreamer, Set, Streamer};

fn most_similar_to<'a>(
    items: impl Iterator<Item = &'a str>,
    buffer: &str,
) -> Result<impl Iterator<Item = usize>, Error> {
    let input = items.map(str::to_lowercase).collect::<Vec<_>>();

    let index_cache = input
        .iter()
        .enumerate()
        .map(|(a, b)| (b.as_bytes(), a))
        .collect::<std::collections::HashMap<_, _>>();

    let input = input.iter().sorted().collect::<Vec<_>>();

    let set = Set::from_iter(&input)?;

    let lev = Levenshtein::new_with_limit(buffer, 5, 300000)?;

    let mut stream = set.search_with_state(lev).into_stream();

    let mut results = vec![];

    while let Some((value, score)) = stream.next() {
        let c = value.clone().to_vec();

        if let Some(score) = score {
            if let Some(key) = index_cache.get(&value[..]) {
                results.push((*key, score, c));
            }
        }
    }

    Ok(results
        .into_iter()
        .sorted_by(|(_, a, _), (_, b, _)| b.cmp(a))
        .map(|(key, _, _)| key))
}

#[test]
fn test_most_similar_to() {
    let set = vec![
        "README.md",
        ".cargo",
        ".ds_store",
        ".git",
        ".gitignore",
        "a",
        "cargo.lock",
        "cargo.toml",
        "entitlements.xml",
        "jago",
        "jago.vim",
        "math",
        "poems",
        "src",
        "target",
    ];

    assert_eq!(
        most_similar_to(set.into_iter(), "readme").unwrap().next(),
        Some(0)
    );
}

impl Directory {
    pub fn write_terminal(&mut self) -> Result<Block, Error> {
        if !self.read {
            let directory = std::fs::read_dir(&self.path)?;

            self.read = true;
            self.entries = directory
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect();
        }

        let mut color_picker = ColorPicker::new();

        let entries = self
            .entries
            .iter()
            .filter_map(|path| path.strip_prefix(&self.path).ok().map(PathBuf::from))
            .map(|path| path.display().to_string().to_lowercase())
            .sorted()
            .collect::<Vec<_>>();

        let active = self.selected_index(entries.clone().as_slice());

        Ok(Block::Group(
            [
                Block::Color(Some(Color::Green)),
                Block::Text("-> ".into()),
                Block::Color(None),
                Block::Text(self.buffer.clone()),
                Block::Text("\n\n".into()),
                Block::NewLine,
            ]
            .into_iter()
            .chain(
                entries
                    .clone()
                    .into_iter()
                    .map(|text| Block::Text(text.into()))
                    .enumerate()
                    .flat_map(|(index, block)| {
                        [
                            if Some(index) == active {
                                Block::Group(vec![Block::Active])
                            } else {
                                Block::Empty
                            },
                            Block::Text(format!("{} ", index)),
                            Block::Color(Some(if Some(index) == active {
                                Color::Green
                            } else {
                                color_picker.pick()
                            })),
                            block,
                            Block::Color(None),
                            if Some(index) == active {
                                Block::Group(vec![
                                    Block::Color(Some(Color::Green)),
                                    Block::Text(" <-".into()),
                                    Block::Color(None),
                                ])
                            } else {
                                Block::Empty
                            },
                            Block::Text("\n".into()),
                            Block::NewLine,
                        ]
                        .into_iter()
                    }),
            )
            .chain([Block::Cursor((0, ("-> ".len() + self.buffer.len(), 0)))].into_iter())
            .collect(),
        ))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl Directory {
    pub fn handle(&mut self, event: &Event) -> Result<Option<&Path>, ()> {
        let mut stop = false;

        let mut marker = None;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                stop = true;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.marker = self
                    .marker
                    .map_or(Some(0), |marker| Some((marker + 1) % self.entries.len()));

                self.buffer = self.entries[self.marker.unwrap()].display().to_string();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.marker = self
                    .marker
                    .map_or(Some(0), |marker| Some(marker.checked_sub(1).unwrap_or(0)));

                self.buffer = self.entries[self.marker.unwrap()].display().to_string();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                self.buffer.pop();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let entries = self.get_entries();

                let entries = entries.iter().map(|abc| abc.as_str()).collect_vec();

                marker = self.selected(&entries);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                ..
            }) => {
                self.buffer.push(*code);
            }
            _ => {}
        };

        if stop {
            Err(())
        } else {
            Ok(marker)
        }
    }
}

use crate::color::Color;

pub type Cursor = (usize, (usize, usize));

enum TextStyle {
    Underline,
}

pub enum Block {
    NewLine,
    Text(String),
    Active,
    Inactive,
    Color(Option<Color>),
    Group(Vec<Block>),
    Cursor(Cursor),
    Empty,
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Attribute, SetAttributes},
    style::{Print, ResetColor, SetForegroundColor},
    Command,
};

impl Command for Block {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Block::NewLine => MoveToColumn(0).write_ansi(out),
            Block::Text(text) => Print(text).write_ansi(out),
            Block::Color(Some(color)) => SetForegroundColor(*color).write_ansi(out),
            Block::Color(None) => ResetColor.write_ansi(out),
            Block::Cursor((_, (x, y))) => MoveTo(*x as u16, *y as u16).write_ansi(out),
            Block::Active => SetAttributes(From::from(
                [Attribute::Underlined, Attribute::Bold].as_ref(),
            ))
            .write_ansi(out),
            Block::Inactive => SetAttributes(From::from(Attribute::Reset)).write_ansi(out),
            Block::Group(slice) => slice
                .iter()
                .map(|block| block.write_ansi(out))
                .find(|result| result.is_err())
                .unwrap_or(Ok(())),
            Block::Empty => Ok(()),
        }
    }
}
