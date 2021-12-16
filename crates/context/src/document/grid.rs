///             1113151719212325272931333537394143454749515355
///  01234567891012141618202224262830323436384042444648505254
/// 0INFO jago gathering src/lib.rs:113                      33
/// 1INFO context yes or no? crates/context/src/lib.rs:50    84
/// 2INFO jago why things are the way they are src/lib.rs:187139
/// 3INFO jago 528.453µs elapsed src/lib.rs:49               179
///                            ^
/// 4*                                                       180
/// 10..18, 46..55, 31..124, 149..165
/// * (0,  4) = 33 + 1 + 51 + 1 + 55 + 1 + 40 + 1 = 183
/// ^ (26, 3) = 33 + 1 + 51 + 1 + 55 + 1 + 26 = 168
///

#[cfg(test)]
static TEST_GRID: &'static str = "INFO jago gathering src/lib.rs:113
INFO context yes or no? crates/context/src/lib.rs:50
INFO jago why things are the way they are src/lib.rs:187
INFO jago 528.453µs elapsed src/lib.rs:49
";

#[test]
fn test_position_from_point() {
    let tests = vec![
        ((0, 0), 0, 'I'),
        ((1, 0), 1, 'N'),
        ((26, 3), 171, 'd'),
        ((0, 4), 188, '\n'),
    ];

    dbg!(TEST_GRID.len());

    for (input, want, value) in tests {
        let got = position_from_point(TEST_GRID.as_bytes(), dbg!(input));
        assert_eq!(got, want);
        assert_eq!(TEST_GRID.as_bytes()[got] as char, value);
    }
}

fn position_from_point(source: &[u8], (x, y): (usize, usize)) -> usize {
    let mut front = 0;

    let mut dy = 0;
    let mut dx = 0;
    dbg!(source.len());
    while front < source.len() {
        if dy == y && dx == x {
            break;
        } else if dy == y {
            dx += 1;
        } else if source[front] == b'\n' {
            dy += 1;
        }

        front += 1;
    }

    front
}
