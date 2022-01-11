pub struct Buffer {
    bytes: Vec<u8>,
    cursor: usize,
}

impl Buffer {
    pub fn forward_graphemes(&self, (x, y): (u16, u16), count: usize) -> (u16, u16) {
        let buffer = self.bytes.as_slice();

        let mut next = input
            .get(self.z()..)
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .take(count)
            .fold(self.clone(), |cursor, grapheme| {
                let mut next = cursor;

                next.2 += grapheme.len();
                next.0 += grapheme.len();

                if "\n" == grapheme {
                    next.1 += 1;
                    next.0 = 0;
                }

                next
            });

        if next.current(input) == "\n" {
            next.2 += 1;
            next.1 += 1;
            next.0 = 0;
        }

        next
    }
}

#[test]
fn forward_graphemes() {
    let buffer = include_str!("../edit");

    macro_rules! assert_ {
        ($from:expr, $steps:expr,  $to:expr, $want:expr) => {
            let from: Buffer = $from.into();
            let to = from.forward_graphemes(buffer, $steps);
            let got = to.current(buffer);
            assert_eq!(
                to,
                $to.into(),
                "{:?} -> {} = got {:?} want {:?} {:?}",
                $from,
                $steps,
                to,
                $to,
                got
            );
            assert_eq!(
                got, $want,
                "{:?} -> {} = got {:?} want {:?}",
                $from, $steps, got, $want
            );
        };
    }

    let tests = vec![
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 1, (2, 0, 2), "J"),
        ((2, 0, 2), 1, (3, 0, 3), "a"),
        ((3, 0, 3), 1, (4, 0, 4), "g"),
        ((4, 0, 4), 1, (5, 0, 5), "o"),
        ((5, 0, 5), 1, (0, 1, 7), "\n"),
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 2, (3, 0, 3), "a"),
        ((3, 0, 3), 3, (0, 1, 7), "\n"),
        ((0, 1, 7), 4, (3, 2, 11), "C"),
        ((3, 2, 10), 5, (8, 2, 15), "e"),
    ];

    for (from, steps, to, want) in tests {
        assert_!(from, steps, to, want);
    }
}
