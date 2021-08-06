pub struct DropCall<F: FnMut()> {
    handler: F,
}

impl<F: FnMut()> DropCall<F> {
    pub fn handle(handler: F) -> Self {
        Self { handler }
    }
}

impl<F: FnMut()> Drop for DropCall<F> {
    fn drop(&mut self) {
        (self.handler)();
    }
}

pub fn log_duration<'a>(key: &'static str) -> DropCall<impl FnMut()> {
    let start = std::time::Instant::now();

    DropCall::handle(move || {
        log::info!("{} -> {:?}", key, start.elapsed());
    })
}
