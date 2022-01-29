use std::path::Path;

pub struct Resource<'a>(&'a Path);

impl<'a> From<&'a Path> for Resource<'a> {
    fn from(path: &'a Path) -> Self {
        Resource(path)
    }
}

use std::fmt;

use crossterm::{style::Print, Command};

impl Command for Resource<'_> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        Print(format!("{}", self.0.display())).write_ansi(out)
    }
}

use crate::traits::Viewer;

impl Viewer for Resource<'_> {}
