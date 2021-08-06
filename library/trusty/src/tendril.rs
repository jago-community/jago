author::error!(WrongFormat, OutOfRange);

use crate::instrument;

use std::cell::RefCell;

use tendril::{
    fmt::{Bytes, UTF8},
    Tendril,
};

#[derive(Debug)]
pub struct Input<'a> {
    reader: RefCell<untrusted::Reader<'a>>,
    buffer: Tendril<Bytes>,
    cursor: usize,
    targets: Targets,
}

bitflags::bitflags! {
    struct Targets: u32 {
        const BYTES = 0b00000000;
        const UTF8 = 0b00000001;
    }
}

impl<'a> nom::AsBytes for Input<'a> {
    fn as_bytes(&self) -> &[u8] {
        instrument::log_duration("fn as_bytes");

        let mut reader = self.reader.borrow_mut();

        if !reader.at_end() {
            reader.skip_to_end();
        }

        self.buffer.as_bytes()
    }
}

use nom::{Compare, CompareResult};

impl<'a> Compare<&'a [u8]> for Input<'a> {
    fn compare(&self, other: &'a [u8]) -> CompareResult {
        self.buffer.as_ref().compare(other)
    }

    fn compare_no_case(&self, other: &'a [u8]) -> CompareResult {
        self.buffer.as_ref().compare_no_case(other)
    }
}

// use std::str;

impl<'a> Compare<&'a str> for Input<'a> {
    fn compare(&self, other: &'a str) -> CompareResult {
        if !self.targets.contains(Targets::UTF8) {
            CompareResult::Error
        } else {
            unsafe {
                self.buffer
                    .reinterpret_view_without_validating::<UTF8>()
                    .as_ref()
                    .compare(other)
            }

            // unsafe { str::from_utf8_unchecked(self.buffer.as_ref()).compare(other) }
        }
    }

    fn compare_no_case(&self, other: &'a str) -> CompareResult {
        if !self.targets.contains(Targets::UTF8) {
            CompareResult::Error
        } else {
            unsafe {
                self.buffer
                    .reinterpret_view_without_validating::<UTF8>()
                    .as_ref()
                    .compare_no_case(other)
            }

            // unsafe { str::from_utf8_unchecked(self.buffer.as_ref()).compare_no_case(other) }
        }
    }
}

use nom::FindSubstring;

impl<'a> FindSubstring<&'a [u8]> for Input<'a> {
    fn find_substring(&self, substr: &'a [u8]) -> Option<usize> {
        self.buffer.as_ref().find_substring(substr)
    }
}

impl<'a> FindSubstring<&'a str> for Input<'a> {
    fn find_substring(&self, substr: &'a str) -> Option<usize> {
        self.buffer_as_utf8()?.as_ref().find_substring(substr)
    }
}

use nom::FindToken;

impl<'a> FindToken<u8> for Input<'a> {
    fn find_token(&self, token: u8) -> bool {
        //compile_error!("do this")
        unimplemented!()
    }
}

impl<'a> FindToken<char> for Input<'a> {
    fn find_token(&self, token: char) -> bool {
        //compile_error!("do this")
        unimplemented!()
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(input: &'a str) -> Self {
        let input = untrusted::Input::from(input.as_bytes());

        Self {
            reader: std::cell::RefCell::new(untrusted::Reader::new(input)),
            buffer: Tendril::new(),
            cursor: 0,
            targets: Targets::all(),
        }
    }
}

impl<'a> Input<'a> {
    fn buffer_as_utf8(&self) -> Option<&Tendril<UTF8>> {
        if self.targets.contains(Targets::UTF8) {
            let buffer = unsafe { self.buffer.reinterpret_view_without_validating::<UTF8>() };

            Some(buffer)
        } else {
            None
        }
    }

    pub fn get(&mut self, range: std::ops::Range<usize>) -> Result<Tendril<Bytes>, Error> {
        instrument::log_duration("Input::get");

        let delta = range
            .end
            .checked_sub(self.cursor)
            .map_or(Err(Error::OutOfRange), Ok)?;

        if range.end > self.cursor {
            let mut reader = self.reader.borrow_mut();

            let input = reader.read_bytes(delta).map_err(|_| Error::OutOfRange)?;

            self.buffer
                .try_push_bytes(input.as_slice_less_safe())
                .map_err(|_| Error::WrongFormat)?;
        }

        Ok(self.buffer.subtendril(range.start as u32, delta as u32))
    }
}
