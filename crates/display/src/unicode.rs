use unicode_segmentation::UnicodeSegmentation;

pub trait UnicodeExt: UnicodeSegmentation {
    fn abc(&self) -> String;
}

#[test]
fn test() {
    let input = "abc";

    input.abc();
}
