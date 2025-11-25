//! Contains some code that initializes logging with some predefined configurations.
//!
//! The main item of this module is the [`init_logger`] function. See its documentation for an explanation of how it configures the logging.

use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, layer::SubscriberExt, registry::Registry,
};
use tracing_tree::HierarchicalLayer;

/// Returns a [`HierarchicalLayer`] initialized with a custom configuration.
pub fn init_std_out_log_formatter() -> HierarchicalLayer {
    HierarchicalLayer::default()
        .with_ansi(true)
        .with_indent_lines(true)
        .with_indent_amount(4)
        .with_deferred_spans(true)
}

/// Returns a [`EnvFilter`] initialized with a custom configuration.
///
/// The filter makes it possible to set the log level and the filtering strategy through the `RUST_LOG` environment variable, as explained [here][EnvFilter]. By default, all the events whose [`Level`][tracing::Level] is [`INFO`][tracing::Level::INFO] or higher are logged.
pub fn init_env_filter() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
}

/// Sets up the logging.
///
/// # Details
/// The logs are printed to the terminal. The log level and filtering strategy can be set through the `RUST_LOG` environment variable, as explained [here][EnvFilter]. By default, all the events whose [`Level`][tracing::Level] is [`INFO`][tracing::Level::INFO] or higher are logged.
pub fn init_logger() {
    let subscriber = Registry::default()
        .with(init_env_filter())
        .with(init_std_out_log_formatter());
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
