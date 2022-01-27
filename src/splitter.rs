pub struct Splitter<Buffer> {
    buffer: Box<dyn Iterator<Item = Cow<'a, str>>>,
}
