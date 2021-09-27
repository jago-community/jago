book::error!(Incomplete);

pub enum Context {
    Content,
    Handle,
    Background,
}

impl Context {
    pub const CONTENT: Self = Self::Content;
    pub const HANDLE: Self = Self::Handle;
    pub const BACKGROUND: Self = Self::Background;
}

pub type Keys<Kind> = crdts::GSet<Kind>;

pub fn handle<Kind: Ord>(context: Context, input: Kind) -> Result<(), Error> {
    match context {
        Context::Content => {
            keys.insert(input);
        }
        Context::Background | Context::Handle => {
            // ...
        }
    };

    Ok(())
}
