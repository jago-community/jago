author::error!(OutOfBounds, NotUtf8);

use either::Either;

use crate::instrument;

pub struct Input<'a> {
    inner: &'a [u8],
    //inner: Either<&'a [u8], Vec<u8>>,
    cursor: usize,
    possible: Formats,
}

bitflags::bitflags! {
    struct Formats: u32 {
        const BYTES = 0b00000000;
        const UTF8 = 0b00000001;
    }
}

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(input: &'a [u8]) -> Self {
        Self {
            inner: input,
            cursor: 0,
            possible: Formats::all(),
        }
    }
}

impl<'a> From<Vec<u8>> for Input<'a> {
    fn from(input: Vec<u8>) -> Self {
        Self {
            inner: input.as_ref(),
            cursor: 0,
            possible: Formats::all(),
        }
    }
}

use std::ops::Range;

impl<'a> Input<'a> {
    fn inner_len(&self) -> usize {
        self.inner.len()
    }

    fn inner_take(&mut self, difference: usize) -> Result<(), Error> {
        let new_cursor = self
            .cursor
            .checked_add(difference)
            .ok_or(Error::OutOfBounds)?;

        let bytes: &[u8] = self
            .inner
            .get(self.cursor..new_cursor)
            .map(From::from)
            .ok_or(Error::OutOfBounds)?;

        for byte in bytes {
            if char::from_u32(*byte as u32).is_none() {
                self.possible.remove(Formats::UTF8);
            }
        }

        self.cursor = new_cursor;

        Ok(())
    }

    fn inner_take_all(&mut self) -> Result<(), Error> {
        self.inner_take(self.inner.len() - self.cursor)
    }

    fn as_str(&'a self) -> Result<&'a str, Error> {
        if self.possible.contains(Formats::UTF8) {
            Ok(unsafe { std::str::from_utf8_unchecked(self.as_bytes()) })
        } else {
            Err(Error::NotUtf8)
        }
    }

    fn as_slice(&self) -> Option<&'a [u8]> {
        self.inner.get(..)
    }
}

use nom::AsBytes;

#[test]
fn test_as_bytes() {
    let mut input = Input::from(&b"fn as_bytes"[..]);

    let (want, got) = (b"", input.as_bytes());
    assert_eq!(want, got);

    input.inner_take_all().unwrap();

    let (want, got) = (b"fn as_bytes", input.as_bytes());
    assert_eq!(want, got);
}

impl<'a> AsBytes for Input<'a> {
    fn as_bytes(&self) -> &[u8] {
        instrument::log_duration("fn as_bytes");

        self.inner.get(..self.cursor).unwrap_or(&[])
    }
}

use nom::{Compare, CompareResult};

#[test]
fn test_compare() {
    let mut input = Input::from(&b"fn as_bytes"[..]);

    input.inner_take(input.inner_len() / 2).unwrap();

    assert_eq!(input.compare(&b"Hello, world!"[..]), CompareResult::Error);

    assert_eq!(
        input.compare(&b"fn as_bytes"[..]),
        CompareResult::Incomplete
    );

    input.inner_take_all().unwrap();

    assert_eq!(input.compare(&b"fn as_bytes"[..]), CompareResult::Ok);

    let mut input = Input::from(&b"fn as_bytes"[..]);

    input.inner_take(input.inner_len() / 2).unwrap();

    assert_eq!(input.compare("Hello, world!"), CompareResult::Error);

    assert_eq!(input.compare("fn as_bytes"), CompareResult::Incomplete);

    input.inner_take_all().unwrap();

    assert_eq!(input.compare("fn as_bytes"), CompareResult::Ok);
}

impl<'a> Compare<&'a [u8]> for Input<'a> {
    fn compare(&self, other: &'a [u8]) -> CompareResult {
        self.as_bytes().compare(other)
    }

    fn compare_no_case(&self, other: &'a [u8]) -> CompareResult {
        self.as_bytes().compare_no_case(other)
    }
}

impl<'a> Compare<&'a str> for Input<'a> {
    fn compare(&self, other: &'a str) -> CompareResult {
        let buffer = match self.as_str() {
            Ok(buffer) => buffer,
            Err(_) => return CompareResult::Error,
        };

        buffer.compare(other)
    }

    fn compare_no_case(&self, other: &'a str) -> CompareResult {
        let buffer = match self.as_str() {
            Ok(buffer) => buffer,
            Err(_) => return CompareResult::Error,
        };

        buffer.compare_no_case(other)
    }
}

use nom::ExtendInto;

impl<'a> ExtendInto for Input<'a> {
    type Item = u8;
    type Extender = Vec<u8>;

    fn new_builder(&self) -> Self::Extender {
        self.inner.new_builder()
    }

    fn extend_into(&self, other: &mut Vec<u8>) {
        self.inner.extend_into(other);
    }
}

struct InputBuilder {
    input: Vec<u8>,
    possible: Formats,
}

use nom::FindSubstring;

impl<'a> FindSubstring<&'a [u8]> for Input<'a> {
    fn find_substring(&self, input: &'a [u8]) -> Option<usize> {
        self.as_bytes().find_substring(input)
    }
}

impl<'a> FindSubstring<&'a str> for Input<'a> {
    fn find_substring(&self, input: &'a str) -> Option<usize> {
        self.as_str()
            .ok()
            .map_or(None, |slice| slice.find_substring(input))
    }
}

use nom::FindToken;

impl<'a> FindToken<u8> for Input<'a> {
    fn find_token(&self, token: u8) -> bool {
        self.as_bytes().find_token(token)
    }
}

impl<'a> FindToken<char> for Input<'a> {
    fn find_token(&self, token: char) -> bool {
        self.as_str().map_or(false, |slice| slice.find_token(token))
    }
}

use nom::{InputIter, Needed};

use std::{
    iter::{Copied, Enumerate},
    slice::Iter,
};

impl<'a> InputIter for Input<'a> {
    type Item = u8;

    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.as_slice().unwrap_or(&[]).iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.iter_elements().position(|b| predicate(b))
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.cursor >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.cursor))
        }
    }
}
