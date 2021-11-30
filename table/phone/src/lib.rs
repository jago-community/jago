use mobile_entry_point::mobile_entry_point;

#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("phone"),
    );
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    simple_logger::SimpleLogger::new().init().unwrap();
}

#[mobile_entry_point]
fn main() {
    init_logging();

    interface::handle([].into_iter(),vec![]).unwrap();
}
