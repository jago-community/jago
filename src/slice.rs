#[derive(Default)]
pub struct Slice<'a> {
    bytes: &'a [u8],
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Reference<'a>(usize, (usize, usize), std::marker::PhantomData<&'a ()>);

#[test]
fn slice_graphemes() {
    let bytes = include_bytes!("../poems/eltheridge-knight/haiku/1");

    let wants = vec![
        ((0, (0, 0)), "E"),
        ((1, (1, 0)), "a"),
        ((2, (2, 0)), "s"),
        ((3, (3, 0)), "t"),
        ((4, (4, 0)), "e"),
        ((5, (5, 0)), "r"),
        ((6, (6, 0)), "n"),
        ((7, (7, 0)), " "),
        ((8, (8, 0)), "g"),
        ((9, (9, 0)), "u"),
        ((10, (10, 0)), "a"),
        ((11, (11, 0)), "r"),
        ((12, (12, 0)), "d"),
        ((13, (13, 0)), " "),
        ((14, (14, 0)), "t"),
        ((15, (15, 0)), "o"),
        ((16, (16, 0)), "w"),
        ((17, (17, 0)), "e"),
        ((18, (18, 0)), "r"),
        // ((19, (19, 0)), "\n"),
        ((20, (0, 1)), "g"),
        ((21, (1, 1)), "l"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let slice = Slice { bytes };

    let gots = slice
        .graphemes()
        .take(21)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let gots = slice
        .graphemes_after(Reference::from((0, (0, 0))))
        .take(20)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().skip(1), gots);

    let gots = slice
        .graphemes_after(Reference::from((1, (1, 0))))
        .take(19)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().skip(2), gots);

    let gots = slice
        .graphemes_before((22, (2, 1)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.rev(), gots);

    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let wants = vec![
        ((0, (0, 0)), "T"),
        ((1, (1, 0)), "h"),
        ((2, (2, 0)), "e"),
        ((3, (3, 0)), " "),
        ((4, (4, 0)), "N"),
        ((5, (5, 0)), "e"),
        ((6, (6, 0)), "w"),
        ((7, (7, 0)), " "),
        ((8, (8, 0)), "R"),
        ((9, (9, 0)), "e"),
        ((10, (10, 0)), "l"),
        ((11, (11, 0)), "i"),
        ((12, (12, 0)), "g"),
        ((13, (13, 0)), "i"),
        ((14, (14, 0)), "o"),
        ((15, (15, 0)), "n"),
        // ((16, (16, 0)), "\n"),
        ((17, (0, 1)), "\n"),
        ((18, (0, 2)), "T"),
        ((19, (1, 2)), "h"),
        ((20, (2, 2)), "e"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let slice = Slice { bytes };

    let gots = slice
        .graphemes()
        .take(20)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let gots = slice
        .graphemes_before((20, (2, 2)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().rev().skip(1), gots);

    let gots = slice
        .graphemes_after((17, (0, 1)).into())
        .take(3)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().skip(17), gots);
}

#[test]
fn slice_lines() {
    let bytes = include_bytes!("../poems/eltheridge-knight/haiku/1");

    let wants = vec![((20, (0, 1)), "g"), ((52, (0, 2)), "l")]
        .into_iter()
        .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let slice = Slice { bytes };

    let gots = slice
        .line_starts_after((0, (0, 0)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let wants = vec![((52, (0, 2)), "l"), ((20, (0, 1)), "g"), ((0, (0, 0)), "E")]
        .into_iter()
        .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .line_starts_before((74, (22, 2)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants, gots);

    let wants = vec![((25, (5, 1)), "s"), ((57, (5, 2)), "l")]
        .into_iter()
        .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .closest_x_in_y_after((5, (5, 0)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let wants = vec![((25, (5, 1)), "s"), ((5, (5, 0)), "r")]
        .into_iter()
        .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .closest_x_in_y_before((52, (5, 2)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let slice = Slice { bytes };

    let wants = vec![
        ((17, (0, 1)), "\n"),
        ((18, (0, 2)), "T"),
        ((57, (0, 3)), "T"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .line_starts_after((5, (5, 0)).into())
        .take(3)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let wants = vec![
        ((57, (0, 3)), "T"),
        ((18, (0, 2)), "T"),
        ((17, (0, 1)), "\n"),
        ((0, (0, 0)), "T"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .line_starts_before((105, (0, 4)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants, gots);

    let wants = vec![((17, (0, 1)), "\n"), ((23, (5, 2)), "o")]
        .into_iter()
        .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .closest_x_in_y_after((5, (5, 0)).into())
        .take(2)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let wants = vec![
        ((23, (5, 2)), "o"),
        ((17, (0, 1)), "\n"),
        ((5, (5, 0)), "e"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let gots = slice
        .closest_x_in_y_before((62, (5, 3)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants, gots);
}

impl<'a> From<&'a [u8]> for Slice<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> From<(usize, (usize, usize))> for Reference<'a> {
    fn from((index, coordinates): (usize, (usize, usize))) -> Self {
        Self(index, coordinates, Default::default())
    }
}

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

impl<'a> Slice<'a> {
    pub fn get(&'a self, reference: Reference) -> Option<&'a str> {
        self.bytes
            .get(reference.index()..)
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .next()
    }

    pub fn graphemes(&'a self) -> impl Iterator<Item = Reference> {
        self.grapheme_span((0, (0, 0)).into(), self.bytes.len())
            .into_iter()
    }

    pub fn grapheme_span(
        &'a self,
        reference: Reference,
        span: usize,
    ) -> impl Iterator<Item = Reference> {
        self.bytes
            .get(reference.index()..reference.index() + span)
            .into_iter()
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .flat_map(|slice| slice.graphemes(true))
            .scan(reference.layout(), |(position, coordinates), grapheme| {
                let current = Some(Reference::from((*position, *coordinates)));

                *position += grapheme.len();

                match grapheme {
                    "\n" => {
                        coordinates.0 = 0;
                        coordinates.1 += 1;
                    }
                    _ => {
                        coordinates.0 += grapheme.len();
                    }
                };

                current
            })
            .enumerate()
            .batching(|it| {
                let (index, next) = it.next()?;

                match (index > 0, self.bytes.get(next.index())) {
                    (true, Some(b'\n')) => it.next().map(|(_, next)| next),
                    _ => Some(next),
                }
            })
    }

    pub fn graphemes_after(&self, reference: Reference) -> impl Iterator<Item = Reference> {
        self.grapheme_span(reference.clone(), self.bytes.len() - reference.index())
            .skip(1)
    }

    pub fn graphemes_before(&self, reference: Reference) -> impl Iterator<Item = Reference> {
        self.bytes
            .get(..reference.index())
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .into_iter()
            .flat_map(|slice| slice.graphemes(true))
            .rev()
            .scan(reference.layout(), |(index, coordinates), word| {
                *index = index.checked_sub(word.len())?;

                if let Some(next) = coordinates.0.checked_sub(word.len()) {
                    coordinates.0 = next;
                } else {
                    coordinates.1 = coordinates.1.checked_sub(1)?;

                    coordinates.0 = *index
                        - self
                            .bytes
                            .get(..*index)
                            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
                            .into_iter()
                            .flat_map(|as_str| as_str.grapheme_indices(true))
                            .rev()
                            .find_map(|(index, grapheme)| {
                                if index == 0 {
                                    Some(0)
                                } else if grapheme == "\n" {
                                    Some(index + 1)
                                } else {
                                    None
                                }
                            })?;
                }

                Some(Reference::from((*index, *coordinates)))
            })
            .batching(|it| {
                let next = it.next()?;

                match self.bytes.get(next.index()) {
                    Some(b'\n') if next.coordinates().0 > 0 => it.next(),
                    _ => Some(next),
                }
            })
    }

    pub fn line_starts_after(&self, reference: Reference) -> impl Iterator<Item = Reference> {
        let start = reference.index();

        self.bytes
            .get(start..self.bytes.len())
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .into_iter()
            .flat_map(|slice| slice.split_word_bounds())
            .scan(reference.layout(), |(index, coordinates), word| {
                let current = Reference::from((*index, *coordinates));

                *index += word.len();

                match word {
                    "\n" => {
                        coordinates.0 = 0;
                        coordinates.1 += 1;
                    }
                    _ => {
                        coordinates.0 += word.len();
                    }
                };

                Some((word, current))
            })
            .batching(move |it| {
                it.find(|(_, reference)| reference.index() > start && reference.x() == 0)
                    .map(|(_, reference)| reference)
            })
    }

    pub fn line_starts_before(&self, reference: Reference) -> impl Iterator<Item = Reference> {
        let stop = reference.index();

        self.bytes
            .get(..stop)
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .into_iter()
            .flat_map(|slice| slice.split_word_bounds())
            .rev()
            .scan(reference.layout(), |(index, coordinates), word| {
                let current = Reference::from((*index, *coordinates));

                *index = index.checked_sub(word.len())?;

                match word {
                    "\n" => {
                        coordinates.1 -= 1;

                        coordinates.0 = *index
                            - self
                                .bytes
                                .get(..*index)
                                .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
                                .into_iter()
                                .flat_map(|as_str| as_str.grapheme_indices(true))
                                .rev()
                                .find_map(|(index, grapheme)| {
                                    if index == 0 {
                                        Some(0)
                                    } else if grapheme == "\n" {
                                        Some(index + 1)
                                    } else {
                                        None
                                    }
                                })?;
                    }
                    _ => {
                        coordinates.0 = coordinates.0.checked_sub(word.len())?;
                    }
                };

                Some((word, current))
            })
            .batching(move |it| {
                it.find(|(word, next)| {
                    next.x() == 0 && next.index() < stop || next.index() - word.len() == 0
                })
                .map(|(word, reference)| {
                    if reference.x() == 0 {
                        reference
                    } else {
                        (
                            reference.index() - word.len(),
                            (reference.x() - word.len(), reference.y()),
                        )
                            .into()
                    }
                })
            })
    }

    pub fn closest_x_in_y_after(&self, reference: Reference) -> impl Iterator<Item = Reference> {
        let target = reference.x();

        self.line_starts_after(reference)
            .flat_map(move |reference| {
                self.grapheme_span(reference.clone(), self.bytes.len() - reference.index())
                    .find_map(|reference| {
                        if reference.x() == target {
                            Some(reference)
                        } else if self.get(reference.clone()) == Some("\n") {
                            if reference.x() > 0 {
                                Some(
                                    (reference.index() - 1, (reference.x() - 1, reference.y()))
                                        .into(),
                                )
                            } else {
                                Some(reference)
                            }
                        } else {
                            None
                        }
                    })
                    .into_iter()
            })
    }

    pub fn closest_x_in_y_before(&self, input: Reference) -> impl Iterator<Item = Reference> {
        let target_x = input.x();
        let target_y = input.y();

        self.line_starts_before(input.clone())
            .skip_while(move |reference| reference.y() == target_y)
            .flat_map(move |reference| {
                self.grapheme_span(reference.clone(), self.bytes.len() - reference.index())
                    .find_map(|reference| {
                        if reference.x() == target_x {
                            Some(reference)
                        } else if self.get(reference.clone()) == Some("\n") {
                            if reference.x() > 0 {
                                Some(
                                    (reference.index() - 1, (reference.x() - 1, reference.y()))
                                        .into(),
                                )
                            } else {
                                Some(reference)
                            }
                        } else {
                            None
                        }
                    })
                    .into_iter()
            })
    }
}

impl<'a> Reference<'a> {
    pub fn index(&self) -> usize {
        self.0
    }

    pub fn coordinates(&self) -> (usize, usize) {
        self.1
    }

    pub fn x(&self) -> usize {
        self.1 .0
    }

    pub fn y(&self) -> usize {
        self.1 .1
    }

    pub fn layout(&self) -> (usize, (usize, usize)) {
        (self.0, self.1)
    }
}
