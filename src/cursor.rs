pub type Location = (usize, (usize, usize));

#[derive(Default, Debug, PartialEq)]
pub struct Cursor {
    front: Location,
    back: Location,
}

impl Cursor {
    fn start(&self) -> usize {
        self.front.0
    }

    fn end(&self) -> usize {
        self.back.0
    }

    fn next(&mut self) -> Option<Location> {
        Some(self.front)
    }

    fn next_back(&mut self) -> Option<Location> {
        Some(self.back)
    }
}

#[test]
#[ignore]
fn cursor() {
    let bytes = include_bytes!("../poems/eltheridge-knight/haiku/1");

    let get = |index: usize| crate::buffer::Buffer::from(&bytes[..]).grapheme(index);

    let mut cursor = Cursor::default();

    assert_eq!(get(cursor.start()), Some("T"));

    assert_eq!(
        cursor,
        Cursor {
            front: (1, (1, 0)),
            back: (72, (0, 3))
        }
    );

    let next = cursor.next().unwrap();

    assert_eq!(
        cursor,
        Cursor {
            front: (1, (1, 0)),
            back: (865, (0, 0))
        }
    );
    assert_eq!(get(cursor.start()), Some("h"));
}
