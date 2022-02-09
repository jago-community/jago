use ::{
    crossterm::Command,
    std::{fmt, io::Write},
};

pub trait View: Write {
    type Buffer;

    fn view_ansi<C: Command>(self: &Self, out: ) -> fmt::Result
    where
        Self: Command,
        Self::Buffer: fmt::Write,
    {
        Ok(())
    }
}
