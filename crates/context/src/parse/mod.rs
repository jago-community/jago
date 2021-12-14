mod error;

use error::Error;

#[test]
fn test_question() {
    let input = b"who?";

    question(input).unwrap();
}

use nom::{bytes::streaming::tag, sequence::terminated, IResult};

fn question<'a>(input: &'a str) -> IResult<&'a str, &'a [usize]> {
    terminated(tag("abc"), tag("?"))(input)
}
