use untrusted::Input as UntrustedInput;

pub struct Span {
    position: usize,
    len: usize,
}

pub struct Input<'a> {
    untrusted: UntrustedInput<'a>,
    graphemes: Vec<Span>,
    words: Vec<Span>,
}

// read one byte at a time, updating states at each point. Word, sentences, valid utf8, etc,
// wrapped

#[test]
fn read_into() {
    // ...
}
