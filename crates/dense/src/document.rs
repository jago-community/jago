#[derive(Debug, PartialEq)]
pub struct Document {
    line_endings: Vec<usize>,
}

impl<'a> From<&'a str> for Document {
    fn from(input: &'a str) -> Self {
        Self {
            line_endings: line_endings(input.as_bytes()),
        }
    }
}

#[test]
fn from_str() {
    let input = "abc de fgi hj
kl m no pq rst
uvw xyz";

    let got = Document::from(input);

    let want = Document {
        line_endings: vec![13, 28],
    };

    assert_eq!(got, want);
}

fn line_endings(input: &[u8]) -> Vec<usize> {
    let mut line_endings = vec![];

    let mut position = 0;

    for item in input {
        if item == &b'\n' {
            line_endings.push(position);
        }

        position += 1;
    }

    line_endings
}
