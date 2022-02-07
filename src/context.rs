use ::{
    crdts::{CmRDT, Dot, LWWReg, List},
    std::sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Context {
    clock: Dot<u8>,
    out: Arc<Mutex<List<String, u8>>>,
}

use crate::directives::{Directive, Event, Op};

impl Directive for Context {
    fn handle(&mut self, event: &Event) -> Op {
        log::info!("{:?}", event);

        match event {
            _ => self.handle_common(event),
        }
    }

    fn step(&mut self) {
        self.clock.apply_inc();

        let out = self.out.clone();
        let out = out.lock().unwrap();

        let len = out.len();

        let out_offset = self.out_offset.clone();
        let mut out_offset = out_offset.lock().unwrap();

        out_offset.update(len, self.clock.counter);
    }
}

use ::{
    crossterm::{
        cursor::{MoveTo, MoveToNextLine},
        style::Print,
        terminal::{Clear, ClearType},
        Command,
    },
    itertools::{FoldWhile, Itertools},
    std::fmt,
};

impl Command for Context {
    fn write_ansi(&self, buffer: &mut impl fmt::Write) -> fmt::Result {
        let q = self.out.clone();
        let q = q.lock().map_err(|_| fmt::Error)?;

        let end = q.len();

        let last_some = (0..8)
            .flat_map(|from_end| end.checked_sub(from_end))
            .flat_map(|index| q.position(index));

        MoveTo(0, 0)
            .write_ansi(buffer)
            .and(Clear(ClearType::All).write_ansi(buffer))?;

        let result = last_some
            .map(|item| {
                Print(item)
                    .write_ansi(buffer)
                    .and(MoveToNextLine(1).write_ansi(buffer))
            })
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

use ::once_cell::sync::OnceCell;

static CONTEXT: OnceCell<Context> = OnceCell::new();

pub fn get() -> Result<&'static Context, log::SetLoggerError> {
    let context = CONTEXT.get_or_init(|| {
        let out = List::new();
        let clock = Dot::new(0, 0);

        Context {
            out: Arc::new(Mutex::new(out)),
            out_offset: Arc::new(Mutex::new(LWWReg { val: 0, marker: 0 })),
            clock,
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
            let op = logs.append(
                format!(
                    "{} {} {:?}",
                    record.level(),
                    record.args(),
                    record
                        .file()
                        .and_then(|file| { record.line().map(|line| (file, line)) })
                ),
                0,
            );
            logs.apply(op);
        }
    }

    fn flush(&self) {}
}
