use std::io::Write;

pub trait Loader {
    type Buffer: Write;

    fn load(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>>;
}
