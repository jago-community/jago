#[cfg(target_arch = "wasm32")]
pub fn before() {
    use log::Level;

    console_log::init_with_level(Level::Info);

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn before() {
    use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};

    registry().with(tracing_subscriber::fmt::layer()).init();
}

pub mod prelude {
    #[cfg(target_arch = "wasm32")]
    pub use log::{debug, error, info, trace, warn};

    #[cfg(not(target_arch = "wasm32"))]
    pub use tracing::{debug, error, info, trace, warn};
}
