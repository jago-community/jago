#[derive(Debug, PartialEq)]
pub struct Cursor {
    position: u16,
    x: u16,
    y: u16,
}

impl Cursor {
    fn find_position(&mut self, source: impl AsRef<[u8]>, (x, y): (u16, u16)) {
        if x == self.x && y == self.y {
            return;
        }

        let up = self.y < y || (self.y == y && self.x < x);

        let source = source.as_ref();

        while self.y != y {
            if up {
                match self.position.checked_add(1) {
                    Some(next) if next as usize > source.len() => {
                        break;
                    }
                    Some(next) if source[next as usize] == b'\n' => {
                        self.y += 1;
                        self.x = 0;
                        self.position = next;
                    }
                    Some(next) => {
                        self.position = next;
                    }
                    _ => {
                        break;
                    }
                };
            }
        }

        while self.x != x {
            if up {
                match self.position.checked_add(1) {
                    Some(next) if next as usize > source.len() => {
                        break;
                    }
                    Some(next) => {
                        self.position = next;
                        self.x += 1;
                    }
                    _ => {
                        break;
                    }
                };
            }
        }

        dbg!(source[self.position as usize] as char);
    }
}

#[test]
fn test_find_position() {
    let source = b"abcdefghij
klmnopqrstuvwxyz!@#$%^&
*()-_+=[{]}|<,>./?'
;:~";

    let inputs = vec![
        (
            (0, 0),
            Cursor {
                position: 0,
                x: 0,
                y: 0,
            },
        ),
        (
            (1, 0),
            Cursor {
                position: 1,
                x: 1,
                y: 0,
            },
        ),
        (
            (7, 0),
            Cursor {
                position: 7,
                x: 7,
                y: 0,
            },
        ),
        (
            (0, 1),
            Cursor {
                position: 10,
                x: 0,
                y: 1,
            },
        ),
        (
            (1, 3),
            Cursor {
                position: 57,
                x: 1,
                y: 3,
            },
        ),
    ];

    let mut cursor = Cursor {
        position: 0,
        x: 0,
        y: 0,
    };

    for (target, want) in inputs {
        println!("{:?} {:?} {:?}", cursor, target, want);
        cursor.find_position(source, target);
        assert_eq!(cursor, want);
    }
}
