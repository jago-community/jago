use ::{
    crdts::{CmRDT, List},
    std::{
        fmt,
        sync::{Arc, Mutex},
    },
};

pub struct Context {
    buffer: Arc<Mutex<List<char, u8>>>,
}

use once_cell::sync::OnceCell;

impl Context {
    pub fn get(buffer: &str) -> &'static Self {
        static CONTEXT: OnceCell<Context> = OnceCell::new();

        CONTEXT.get_or_init(move || Context::from(buffer))
    }
}

static DEFAULT_ACTOR: u8 = 0;

impl From<&str> for Context {
    fn from(input: &str) -> Self {
        let mut buffer = List::new();

        for c in input.chars() {
            let op = buffer.append(c, DEFAULT_ACTOR);
            buffer.apply(op);
        }

        Self {
            buffer: Arc::new(Mutex::new(buffer)),
        }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = self.buffer.lock().map_err(|_| fmt::Error)?.read::<String>();

        f.write_str(&buffer)
    }
}
