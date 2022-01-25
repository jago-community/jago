use std::collections::{HashMap, HashSet};

use ndarray::{arr2, Array2};

pub struct Grid<'a> {
    parts: Vec<&'a str>,
    part_map: HashMap<usize, &'a str>,
    cells: Array2<usize>,
}

use std::borrow::Cow;

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

impl<'a> Grid<'a> {
    pub fn new((x, y): (u16, u16), input: Cow<'a, str>) -> Self {
        let input = &input[..];

        let (parts, part_map, cells) = input[..].split_word_bound_indices().fold(
            (
                HashSet::new(),
                HashMap::new(),
                Array2::zeros([x as usize, y as usize]),
            ),
            |(mut parts, mut part_map, mut cells), (index, part)| {
                if !parts.contains(&part) {
                    parts.insert(part);
                    part_map.insert(parts.len(), part);
                }

                (parts, part_map, cells)
            },
        );

        Self {
            parts: parts.into_iter().collect_vec(),
            part_map,
            cells,
        }

        //let cells = input.split_word_bounds().fold(
        //,
        //|sofar, word| {
        //// ...
        //sofar
        //},
        //);

        //Self { blocks, cells }
    }
}
