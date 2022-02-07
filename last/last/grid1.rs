use std::collections::{HashMap, HashSet};

use ndarray::{Array1, Array2};

pub struct Grid<'a> {
    parts: Vec<&'a str>,
    part_map: HashMap<usize, Cow<'a, str>>,
    cells: Array2<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Shape {0}")]
    Shape(#[from] ndarray::ShapeError),
}

use std::borrow::Cow;

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

impl<'a> Grid<'a> {
    pub fn new((x, y): (u16, u16), input: &'a str) -> Result<Self, Error> {
        let words = input.split_word_bounds();

        let mut parts = HashSet::new();
        let mut part_map = HashMap::new();
        let mut cells = Array2::zeros([x as usize, y as usize]);

        let mut row = Vec::<usize>::with_capacity(x as usize);

        for word in words {
            if word == "\n" {
                let row = Array1::from(row.clone());
                cells.push_row(row.view())?;
            } else {
                parts.insert(word);
                part_map.insert(parts.len(), Cow::Borrowed(word));

                row.extend_from_slice(
                    &std::iter::repeat(parts.len())
                        .take(parts.len())
                        .collect_vec(),
                );
            }
        }

        Ok(Self {
            parts: parts.into_iter().collect_vec(),
            part_map,
            cells,
        })
    }
}

#[test]
fn test_grid() -> Result<(), Error> {
    let bytes = include_str!("../poems/chris-abani/the-new-religion");

    let grid = Grid::new((52, 23), bytes)?;

    //assert_eq!(bytes, grid.read());

    Ok(())
}
