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

use std::borrow::Cow;

use itertools::Itertools;

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
    ]
    .into_iter()
    .map(|a| Cow::from(a))
    .collect_vec();

    let directory = Directory {
        path: PathBuf::new(),
        read: true,
        entries: entries
            .clone()
            .into_iter()
            .map(|a| PathBuf::from(&a[..]))
            .collect_vec(),
        marker: None,
        buffer: String::from("readme"),
    };

    assert_eq!(directory.selected(), Some(Path::new("README.md")));
}

impl Directory {
    fn set_entries(&mut self) -> Result<(), Error> {
        let directory = std::fs::read_dir(&self.path)?;

        self.read = true;
        self.entries = directory
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter_map(|path| path.strip_prefix(&self.path).ok().map(PathBuf::from))
            .sorted_by(|a, b| {
                a.as_os_str()
                    .to_ascii_lowercase()
                    .cmp(&b.as_os_str().to_ascii_lowercase())
            })
            .collect();

        Ok(())
    }

    fn get_entries<'a>(&self) -> Vec<Cow<'_, str>> {
        self.entries
            .iter()
            .map(|path| Cow::from(path.display().to_string()))
            .collect()
    }

    fn selected_index<'a>(&self) -> Option<usize> {
        let entries = self.get_entries();

        let mut index = None;

        if self.buffer.len() > 0 {
            let mut sorted_entry_indices = crate::order::similar(&self.buffer, entries.iter());

            if let Some(first) = sorted_entry_indices.next() {
                index = Some(first);
            }
        } else if let Some(marker) = self.marker {
            index = Some(marker);
        }

        index
    }

    fn selected<'a>(&self) -> Option<&Path> {
        self.selected_index()
            .iter()
            .flat_map(|index| self.entries.get(*index))
            .map(|item| item.as_path())
            .next()
    }
}

use crate::color::ColorPicker;

impl Directory {
    pub fn write_terminal(&mut self) -> Result<Block, Error> {
        if !self.read {
            self.set_entries()?;
        }

        let mut color_picker = ColorPicker::new();

        let entries = self.get_entries();

        let active = self.selected_index();

        Ok(Block::Group(
            [
                Block::Color(Some(Color::Green)),
                Block::Text("-> ".into()),
                Block::Color(None),
                Block::Text(self.buffer.clone()),
                Block::Text("\n ".into()),
                Block::NewLine,
                Block::Color(Some(Color::Green)),
                Block::Text("-> ".into()),
                Block::Color(None),
                Block::Text(format!("{:?}", self.selected())),
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
                marker = self.selected();
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
