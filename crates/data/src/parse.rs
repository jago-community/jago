#[test]
fn test_position() {
    let a = b"abc defg h
ij klm nop
";

    let input = vec![
        ((0, 0), 0, 'a'),
        ((5, 0), 5, 'e'),
        ((3, 1), 14, 'k'),
        ((0, 2), 21, '\n'),
    ];

    for (input, want_position, want_value) in input {
        let got = position(a, b'\n', dbg!(input)).unwrap();
        assert_eq!(want_position, got.1);
        assert_eq!(want_value, a[got.1] as char);
    }
}

use nom::{
    bytes::complete::{tag, take_till},
    combinator::map,
    multi::count,
    sequence::terminated,
    IResult,
};

///  0123456789
/// 0abc defg h10
///       ^
/// 1ij klm nop21
///     !
/// 2          22
/// *
///
/// ^ = (5, 0) = 5   
/// ! = (4, 1) = 14  
/// * = (0, 2) = 21  
///
/// abc defg h ij klm nop
/// 0123456789
///           0123456789
///                     0
pub fn position(source: &[u8], mark: u8, (x, y): (usize, usize)) -> IResult<&[u8], usize> {
    println!(
        "{}",
        (0..source.len())
            .into_iter()
            .map(|i| i.to_string())
            .collect::<String>()
    );

    println!("{}", String::from_utf8_lossy(source));

    map(
        count(terminated(take_till(|this| this == mark), tag("\n")), y),
        |those: Vec<&[u8]>| {
            those
                .clone()
                .iter()
                .map(|this| String::from_utf8_lossy(this))
                .for_each(|this| {
                    dbg!(this.len());
                    dbg!(this);
                });

            dbg!(x) + dbg!(those.len()) + those.iter().map(|part| part.len()).sum::<usize>()
        },
    )(source)
}
