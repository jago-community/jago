book::error!(Incomplete, std::io::Error, ExpectedField);

#[test]
fn test_index() {
    assert_eq!(2 + 2, 4);
}

#[cfg(feature = "web-sys")]
pub fn index<E>(_input: &web_sys::Node, handle: impl Fn(&str) -> Result<(), E>) -> Result<(), E> {
    handle("hello stranger")
}
