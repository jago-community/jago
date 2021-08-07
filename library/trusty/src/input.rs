author::error!(OutOfBounds, NotUtf8);

use crate::instrument;

#[derive(Debug, Clone)]
pub struct Input<'a> {
    inner: &'a [u8],
    cursor: usize,
    possible: Formats,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WrappedInput {
    inner: Vec<u8>,
    cursor: usize,
    possible: Formats,
}

use std::fmt;

impl fmt::Display for WrappedInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.possible.contains(Formats::UTF8) {
            write!(f, "{}", self.as_str().unwrap_or("<infallible>"))
        } else {
            write!(f, "{:?}", self.as_bytes())
        }
    }
}

bitflags::bitflags! {
    struct Formats: u32 {
        const BYTES = 0b00000000;
        const UTF8 = 0b00000001;
    }
}

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(input: &'a [u8]) -> Self {
        Input {
            inner: input,
            cursor: 0,
            possible: Formats::all(),
        }
    }
}

impl<'a> Input<'a> {
    pub fn wrap(&'a self) -> WrappedInput {
        WrappedInput {
            inner: self.as_bytes().to_vec(),
            cursor: self.cursor,
            possible: self.possible,
        }
    }

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

    fn slice_inner(&self) -> Option<&'a [u8]> {
        self.inner.get(..)
    }
}

impl WrappedInput {
    pub fn as_str<'a>(&'a self) -> Result<&'a str, Error> {
        if self.possible.contains(Formats::UTF8) {
            Ok(unsafe { std::str::from_utf8_unchecked(self.as_bytes()) })
        } else {
            Err(Error::NotUtf8)
        }
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

impl AsBytes for WrappedInput {
    fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

use nom::{Compare, CompareResult};

#[test]
#[ignore]
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
        self.inner.compare(other)
    }

    fn compare_no_case(&self, other: &'a [u8]) -> CompareResult {
        self.inner.compare_no_case(other)
    }
}

impl<'a> Compare<&'a str> for Input<'a> {
    fn compare(&self, other: &'a str) -> CompareResult {
        self.inner.compare(AsBytes::as_bytes(other))
    }

    fn compare_no_case(&self, other: &'a str) -> CompareResult {
        self.inner.compare_no_case(AsBytes::as_bytes(other))
    }
}

impl<'a> Compare<&'a str> for &'a Input<'a> {
    fn compare(&self, other: &'a str) -> CompareResult {
        self.inner.compare(AsBytes::as_bytes(other))
    }

    fn compare_no_case(&self, other: &'a str) -> CompareResult {
        self.inner.compare_no_case(AsBytes::as_bytes(other))
    }
}

use nom::ExtendInto;

impl<'a> ExtendInto for &'a Input<'a> {
    type Item = u8;
    type Extender = Vec<u8>;

    fn new_builder(&self) -> Self::Extender {
        dbg!("new_builder");
        self.inner.new_builder()
    }

    fn extend_into(&self, other: &mut Vec<u8>) {
        dbg!("extend_into");
        self.inner.extend_into(other);
    }
}

use nom::FindSubstring;

impl<'a> FindSubstring<&'a [u8]> for Input<'a> {
    fn find_substring(&self, input: &'a [u8]) -> Option<usize> {
        self.inner.find_substring(input)
    }
}

impl<'a> FindSubstring<&'a str> for Input<'a> {
    fn find_substring(&self, input: &'a str) -> Option<usize> {
        self.inner.find_substring(AsBytes::as_bytes(input))
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

impl<'a> InputIter for &'a Input<'a> {
    type Item = u8;

    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.slice_inner().unwrap_or(&[]).iter_elements()
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

use nom::InputLength;

impl<'a> InputLength for &'a Input<'a> {
    fn input_len(&self) -> usize {
        self.inner.len()
    }
}

use nom::InputTake;

impl<'a> InputTake for Input<'a> {
    fn take(&self, next_cursor: usize) -> Self {
        let next_bytes = self.inner.take(next_cursor);

        //let next_bytes: &[u8] = self
        //.inner
        //.get(self.cursor..next_cursor)
        //.map(From::from)
        //.unwrap_or(&[]);

        let mut next_possible = self.possible;

        for byte in next_bytes {
            if char::from_u32(*byte as u32).is_none() {
                next_possible.remove(Formats::UTF8);
            }
        }

        Self {
            inner: self.inner,
            cursor: next_cursor,
            possible: next_possible,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (
            self.take(count),
            self.cursor
                .checked_sub(count)
                .or(Some(0))
                .map(|next_cursor| Self {
                    inner: self.inner.get(count..).unwrap_or(&[]),
                    cursor: next_cursor,
                    possible: self.possible,
                })
                .unwrap(),
        )
    }
}

impl<'a> Input<'a> {
    pub fn take_all(&self) -> Self {
        self.take(self.inner.len())
    }
}

use nom::{
    error::{ErrorKind, ParseError},
    Err as ErrorWrap, IResult, InputTakeAtPosition,
};

impl<'a> InputTakeAtPosition for Input<'a> {
    type Item = u8;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let bytes = self.as_bytes();

        dbg!("split_at_position");

        match (0..bytes.len()).find(|b| predicate(bytes[*b])) {
            Some(position) => Ok(self.take_split(position)),
            None => Err(ErrorWrap::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        failed_case: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let bytes = self.as_bytes();

        dbg!("split_at_position1");

        match (0..bytes.len()).find(|b| predicate(bytes[*b])) {
            Some(0) => Err(ErrorWrap::Error(E::from_error_kind(
                self.clone(),
                failed_case,
            ))),
            Some(position) => Ok(self.take_split(position)),
            None => Err(ErrorWrap::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let bytes = self.as_bytes();
        dbg!("split_at_position_complete");
        match (0..bytes.len()).find(|b| predicate(bytes[*b])) {
            Some(position) => Ok(self.take_split(position)),
            None => Ok(self.take_split(self.input_len())),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let bytes = self.as_bytes();
        dbg!("split_at_position_complete");
        match (0..bytes.len()).find(|b| predicate(bytes[*b])) {
            Some(0) => Err(ErrorWrap::Error(E::from_error_kind(self.clone(), e))),
            Some(position) => Ok(self.take_split(position)),
            None => {
                if bytes.is_empty() {
                    Err(ErrorWrap::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

use nom::Offset;

impl<'a> Offset for Input<'a> {
    fn offset(&self, second: &Self) -> usize {
        let start = self.inner.as_ptr();
        let stop = second.inner.as_ptr();

        stop as usize - start as usize
    }
}

use nom::ParseTo;

use std::str::FromStr;

impl<'a, R: FromStr> ParseTo<R> for Input<'a> {
    fn parse_to(&self) -> Option<R> {
        self.as_str().ok().and_then(|buffer| buffer.parse().ok())
    }
}

use nom::Slice;

use std::ops::Range;

impl<'a> Slice<Range<usize>> for Input<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        range
            .end
            .checked_sub(self.cursor)
            .map_or(self.clone(), |difference| self.take(difference))
    }
}
