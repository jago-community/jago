pub struct Source<Buffer> {
    buffer: Buffer,
}

use log::{Log, Metadata, Record};

impl<B> Log for Source<B>
where
    B:,
{
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Ok(mut logs) = self.logs.lock() {
            format!(
                "{} {} {:?}\n",
                record.level(),
                record.args(),
                record
                    .file()
                    .and_then(|file| { record.line().map(|line| (file, line)) })
            )
            .chars()
            .map(|ch| logs.append(ch, 0))
            .for_each(|op| logs.apply(op));
        }
    }

    fn flush(&self) {}
}
