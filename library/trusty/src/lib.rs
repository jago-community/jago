#[derive(Debug)]
pub struct Input<'a> {
    source: untrusted::Reader<'a>,
    cursor: usize,
    buffers: Vec<untrusted::Input<'a>>,
    exposed: std::cell::RefCell<Vec<u8>>,
}

impl<'a> nom::AsBytes for Input<'a> {
    fn as_bytes(&self) -> &[u8] {
        log::trace!("combining memory");

        let mut output = match self.exposed.try_borrow_mut() {
            Ok(output) => output,
            Err(_) => return &[],
        };

        if output.len() > 0 {
            output.clear();
        }

        for slice in &self.buffers {
            output.extend_from_slice(slice.as_slice_less_safe());
        }

        output.as_ref()
    }
}

impl<'a> Input<'a> {
    pub fn get(&mut self, range: std::ops::Range<usize>) -> Option<Input<'a>> {
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
            source: untrusted::Reader::new(untrusted::Input::from(input.as_bytes())),
            cursor: 0,
            buffers: vec![],
        }
    }
}
