use ::{crossterm::Command, std::fmt};

pub trait View {
    type Buffer;

    fn view_ansi(self: &Self, out: &mut Self::Buffer) -> fmt::Result
    where
        Self: Command,
        Self::Buffer: fmt::Write,
    {
        self.write_ansi(out)
    }
}
