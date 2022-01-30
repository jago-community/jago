use crossterm::Command;

//pub struct Lense<'a>(Box<dyn Command + 'a>);

//pub trait View<'a> {
//fn view(&self) -> Lense<'a>;
//}

//pub struct Group<'a>(pub Vec<Lense<'a>>);

//impl<'a> Command for Group<'a> {
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//for item in self.0 {
//item.0.write_ansi(out)?;
//}

//Ok(())
//}
//}

//use std::fmt::Display;

//pub struct Encoded<'a, A>(pub &'a A);

//impl<'a, A> Display for &'a Encoded<'_, A>
//where
//A: Display,
//{
//fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//self.0.fmt(out)
//}
//}

//use crossterm::style::Print;

//impl<'a, A> Command for &'a Encoded<'a, A>
//where
//&'a A: Display,
//{
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//Print(&self.0).write_ansi(out)
//}
//}
