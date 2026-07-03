use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub struct LogGuard {
    _file_guard: WorkerGuard,
}

pub fn init(data_dir: &Path) -> std::io::Result<LogGuard> {
    std::fs::create_dir_all(data_dir)?;
    let file_appender = tracing_appender::rolling::daily(data_dir, "imagevue.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();

    Ok(LogGuard { _file_guard: file_guard })
}
