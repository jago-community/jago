use untrusted::Reader;

use std::cell::RefCell;

pub struct Input<'a> {
    source: Reader<'a>,
    cursor: usize,
    buffers: Vec<untrusted::Input<'a>>,
}

use std::ops::Range;

#[test]
#[ignore]
fn test_slice_as_utf8() {
    let mut input = Input::from("Hello again");
    let want = "agai".into();
    let got = input.slice_as_utf8("Hello ".len().."Hello agai".len());
    assert_eq!(got, want);
}

impl<'a> Input<'a> {
    fn slice_as_utf8(&mut self, range: Range<usize>) -> Option<&'a str> {
        if range.end > self.cursor {
            let input = self
                .source
                .read_bytes(range.end - self.cursor)
                .map(Some)
                .map_err(|_| Option::<untrusted::Input>::None);

            if let Some(input) = input.unwrap() {
                self.buffers.push(input);
            }
        }

        unimplemented!()
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(input: &'a str) -> Self {
        Self {
            source: Reader::new(untrusted::Input::from(input.as_bytes())),
            cursor: 0,
            buffers: vec![],
        }
    }
}
