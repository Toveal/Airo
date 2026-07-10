//! Logging setup.
//!
//! Release builds have no console (`windows_subsystem = "windows"`), so
//! anything written to stdout there is lost. Logs go to a file next to the
//! config; debug builds mirror them to stdout as well.

use std::path::Path;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

/// The background writer lives as long as this guard does; drop it and the
/// tail of the log is lost.
static GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init(log_dir: &Path) {
    if GUARD.get().is_some() {
        return;
    }

    if let Err(err) = std::fs::create_dir_all(log_dir) {
        eprintln!("could not create the log directory: {err}");
        return;
    }

    let appender = tracing_appender::rolling::daily(log_dir, "airo.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let _ = GUARD.set(guard);

    let filter = EnvFilter::try_from_env("AIRO_LOG").unwrap_or_else(|_| {
        EnvFilter::new(if cfg!(debug_assertions) {
            "airo_lib=debug,warn"
        } else {
            "airo_lib=info,warn"
        })
    });

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(writer);

    let registry = tracing_subscriber::registry().with(filter).with(file_layer);

    #[cfg(debug_assertions)]
    let registry = registry.with(tracing_subscriber::fmt::layer());

    if registry.try_init().is_err() {
        eprintln!("a tracing subscriber is already installed");
    }
}
