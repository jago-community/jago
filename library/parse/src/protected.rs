use untrusted::{Input, Reader};

pub struct Bytes<'a> {
    reader: Reader<'a>,
    buffer: Option<Input<'a>>,
}

impl<'a> Bytes<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            reader: Reader::new(Input::from(bytes)),
            buffer: None,
        }
    }
}

/*
impl nom::AsBytes for Bytes<'_> {
    fn as_bytes(&self) -> &[u8] {
        self.reader.read_bytes_to_end().as_slice_less_safe()
    }
}
*/
/*

use nom::CompareResult;
use unicode_segmentation::UnicodeSegmentation;

impl nom::Compare<&[u8]> for Bytes<'_> {
    fn compare(&self, t: &[u8]) -> CompareResult {
        if self.untrusted == t {
            CompareResult::Ok
        } else {
            CompareResult::Error
        }
    }

    fn compare_no_case(&self, t: &[u8]) -> CompareResult {
        let untrusted = &self.untrusted[..t.len()];

        if self.possible.contains(Encodings::UTF8) {
            if let (Ok(untrusted), Ok(other)) =
                (std::str::from_utf8(untrusted), std::str::from_utf8(t))
            {
                let mut untrusted = untrusted.graphemes(true);
                let mut other = other.graphemes(true);

                while let (Some(next_untrusted), Some(next_other)) =
                    (untrusted.next(), other.next())
                {
                    if next_untrusted.to_lowercase() != next_other.to_lowercase() {
                        return CompareResult::Error;
                    }
                }

                return CompareResult::Ok;
            }
        }

        CompareResult::Error
    }
}

impl nom::Compare<&str> for Bytes<'_> {
    fn compare(&self, t: &str) -> CompareResult {
        if self.untrusted == t.as_bytes() {
            CompareResult::Ok
        } else {
            CompareResult::Error
        }
    }

    fn compare_no_case(&self, t: &str) -> CompareResult {
        let untrusted = &self.untrusted[..t.len()];

        if self.possible.contains(Encodings::UTF8) {
            if let Ok(untrusted) = std::str::from_utf8(untrusted) {
                let mut untrusted = untrusted.graphemes(true);
                let mut other = t.graphemes(true);

                while let (Some(next_untrusted), Some(next_other)) =
                    (untrusted.next(), other.next())
                {
                    if next_untrusted.to_lowercase() != next_other.to_lowercase() {
                        return CompareResult::Error;
                    }
                }

                return CompareResult::Ok;
            }
        }

        CompareResult::Error
    }
}
*/
