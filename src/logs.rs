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
