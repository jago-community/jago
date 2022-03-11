#[cfg(target_arch = "wasm32")]
pub fn before() {
    use log::Level;

    console_log::init_with_level(Level::Info);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn before() {
    use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};

    let crate_names = workspace::crate_names!();

    let default_filters = crate_names
        .iter()
        .map(|name| [name, "debug"].join("="))
        .collect::<Vec<_>>()
        .join(",");

    registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| default_filters),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub mod prelude {
    #[cfg(target_arch = "wasm32")]
    pub use log::{debug, error, info, trace, warn};

    #[cfg(not(target_arch = "wasm32"))]
    pub use tracing::{debug, error, info, trace, warn};
}
