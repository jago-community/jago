use crdts::{CmRDT, List};

use std::sync::Mutex;

pub struct Context {
    out: Mutex<List<String, u8>>,
}

use crate::directives::{Directive, Event, Op};

impl Directive for Context {
    fn handle(&mut self, event: &Event) -> Op {
        log::info!("{:?}", event);

        match event {
            _ => self.handle_common(event),
        }
    }
}

use ::{
    crossterm::{style::Print, Command},
    itertools::{FoldWhile, Itertools},
    std::fmt,
};

impl Command for Context {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let logs = self.out.lock().map_err(|_| fmt::Error)?;

        let result = logs
            .iter()
            .map(|item| Print(item).write_ansi(out))
            .fold_while(Ok(()), |_, next| {
                if next.is_ok() {
                    FoldWhile::Continue(Ok(()))
                } else {
                    FoldWhile::Done(next)
                }
            });

        result.into_inner()
    }
}

use once_cell::sync::OnceCell;

static CONTEXT: OnceCell<Context> = OnceCell::new();

pub fn get() -> Result<&'static Context, log::SetLoggerError> {
    let context = CONTEXT.get_or_init(|| {
        let out = List::new();

        Context {
            out: Mutex::new(out),
        }
    });

    log::set_logger(context)?;
    log::set_max_level(log::LevelFilter::Info);

    Ok(context)
}

use log::{Log, Metadata, Record};

impl Log for Context {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Ok(mut logs) = self.out.lock() {
            let op = logs.append(format!("{}", record.level()), 0);
            logs.apply(op);
        }
    }

    fn flush(&self) {}
}
