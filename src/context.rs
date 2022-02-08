use ::{
    crdts::{CmRDT, List, MVReg, Map},
    std::sync::{Arc, Mutex},
};

use crate::directives::color_picker::{Color, ColorPicker};

#[derive(Clone)]
pub struct Context {
    buffer: Arc<Mutex<List<char, u8>>>,

    out: Arc<Mutex<List<String, u8>>>,
    out_colors: Arc<Mutex<Map<usize, MVReg<Color, u8>, u8>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Lock")]
    Lock,
}

use crate::directives::{Directive, Event, Op};

impl Directive for Context {
    fn before(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let span = self
            .out
            .clone()
            .lock()
            .map_err(|_| Box::new(&Error::Lock))?
            .len();

        let cache = self.out_colors.clone();
        let mut cache = cache.lock().map_err(|_| Box::new(&Error::Lock))?;

        let mut color_picker = ColorPicker::new();

        let keys_to_add = (0..span)
            .filter(|index| cache.get(&index).val.is_none())
            .collect::<Vec<_>>();

        for index in keys_to_add {
            let write = cache.read_ctx().derive_add_ctx(0);

            let op = cache.update(index, write, |v, a| v.write(color_picker.pick(), a));

            cache.apply(op);
        }

        Ok(())
    }

    fn handle(&mut self, event: &Event) -> Op {
        log::info!("{:?}", event);

        match event {
            _ => self.handle_common(event),
        }
    }
}

use ::{
    crossterm::{
        cursor::{MoveTo, MoveToNextLine},
        style::{Print, SetForegroundColor},
        terminal::{size, Clear, ClearType},
        Command,
    },
    itertools::{FoldWhile, Itertools},
    num_traits::FromPrimitive,
    std::fmt,
};

impl Command for Context {
    fn write_ansi(&self, buffer: &mut impl fmt::Write) -> fmt::Result {
        let (_, height) = size().map_err(|_| fmt::Error)?;

        let q = self.out.clone();
        let q = q.lock().map_err(|_| fmt::Error)?;

        let end = q.len();

        let upper = usize::from_u16(height).ok_or(fmt::Error)?;

        MoveTo(0, height / 2)
            .write_ansi(buffer)
            .and(Clear(ClearType::All).write_ansi(buffer))?;

        let color_cache = self.out_colors.clone();
        let color_cache = color_cache.lock().expect("hell na");

        let colors = (0..upper / 2)
            .flat_map(|from_end| end.checked_sub(from_end))
            .flat_map(|index| {
                let color = color_cache.get(&index);

                color.val
            })
            .flat_map(|set| set.read().val.first().cloned());

        let last_some = (0..upper / 2)
            .flat_map(|from_end| end.checked_sub(from_end))
            .flat_map(|index| q.position(index))
            .zip(colors);

        let result = last_some
            .map(|(item, color)| {
                SetForegroundColor(color)
                    .write_ansi(buffer)
                    .and(Print(item).write_ansi(buffer))
                    .and(MoveToNextLine(1).write_ansi(buffer))
                    .and(SetForegroundColor(Color::Reset).write_ansi(buffer))
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
        let buffer = List::new();
        let out = List::new();
        let colors = Map::new();

        Context {
            buffer: Arc::new(Mutex::new(buffer)),
            out: Arc::new(Mutex::new(out)),
            out_colors: Arc::new(Mutex::new(colors)),
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
