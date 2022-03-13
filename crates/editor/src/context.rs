pub trait Directive {
    fn stop(&self) -> bool;
}

pub trait Handle {
    type Event;
    type Directive: Directive;

    fn handle(&self, event: &Self::Event) -> Self::Directive;
}

use bitflags::bitflags;

bitflags! {
    pub struct Directives: u32 {
        const STOP = 0b00000001;
    }
}

impl Directive for Directives {
    fn stop(&self) -> bool {
        self.contains(Directives::STOP)
    }
}
