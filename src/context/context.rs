use ::{
    crdts::{CmRDT, List, MVReg, Map},
    std::sync::{Arc, Mutex},
};

use crate::colors::Color;

#[derive(Clone)]
pub struct Context {
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

/*
use super::directive::{Directive, Event, Handle};

impl Directive for Context {
    fn before(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let span = self
            .out
            .clone()
            .lock()
            .map_err(|_| Box::new(&Error::Lock))?
            .len();

        let mut cache = self.out_colors.lock().map_err(|_| Box::new(&Error::Lock))?;

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

    fn handle(&mut self, event: &Event) -> Handle {
        log::info!("{:?}", event);

        self.handle_common(event)
    }
}
*/

use ::once_cell::sync::OnceCell;

static CONTEXT: OnceCell<Context> = OnceCell::new();

pub fn get() -> Result<&'static Context, log::SetLoggerError> {
    let context = CONTEXT.get_or_init(|| {
        let out = List::new();
        let colors = Map::new();

        Context {
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
        if let Ok(mut out) = self.out.lock() {
            let op = out.append(
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
            out.apply(op);
        }
    }

    fn flush(&self) {}
}
