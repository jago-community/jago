use std::borrow::Cow;

pub struct Grid<'a> {
    source: Cow<'a, str>,
    dimensions: (usize, usize),
    cursor: (usize, (usize, usize)),
}

impl<'a> Grid<'a> {
    pub fn new(source: &'a str, (x, y): (usize, usize)) -> Self {
        Self {
            source: source.into(),
            dimensions: (x, y),
            cursor: (0, (0, 0)),
        }
    }
}

impl<'a> Grid<'a> {
    pub fn spans(&self) -> impl Iterator<Item = (usize, usize)> {
        vec![].into_iter()
    }

    pub fn read(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.spans()
            .map(|(index, slots)| self.source.get(index..index + slots).unwrap_or("\n").into())
    }
}

#[test]
fn test_grid() {
    let bytes = include_str!("../poems/etheridge-knight/haiku/1");

    assert_eq!(bytes, Grid::new(bytes, (31, 3)).read().collect::<String>());
}
