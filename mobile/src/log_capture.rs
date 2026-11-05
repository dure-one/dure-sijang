use chrono::Utc;
use std::sync::{Arc, OnceLock, RwLock};

/// Format log message with timestamp and level
fn format_log(level: log::Level, target: &str, message: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");

    let level_str = match level {
        log::Level::Error => "ERROR",
        log::Level::Warn => "WARN",
        log::Level::Info => "INFO",
        log::Level::Debug => "DEBUG",
        log::Level::Trace => "TRACE",
    };

    format!("{} [{}] {}: {}", timestamp, level_str, target, message)
}

/// Target prefixes for noisy transitive dependencies (pulled in by eframe/egui's Linux
/// accessibility bridge and dark-mode detection) that spam INFO/DEBUG logs of their own
/// (DBus handshakes, connection polling, etc.) unrelated to app behavior. Capped at WARN
/// regardless of the app's configured level, unless the user explicitly asks for TRACE.
///
/// "tracing::span" / "tracing::event" are `tracing`'s own log-facade fallback targets: since
/// this app never installs a `tracing_subscriber`, any dependency using `tracing` (zbus, its
/// DBus stack, etc.) falls back to emitting through the `log` crate under these fixed target
/// names instead of the crate's own name, which is why they aren't caught by a crate-name match.
const NOISY_TARGET_PREFIXES: &[&str] = &[
    "zbus",
    "atspi",
    "accesskit_unix",
    "ashpd",
    "async_io",
    "async_executor",
    "polling",
    "tracing::span",
    "tracing::event",
];

/// Combined logger that sends logs to both logcat (on Android) and in-app UI capture
struct CombinedLogger {
    level_filter: Arc<RwLock<log::LevelFilter>>,
}

impl log::Log for CombinedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let Ok(filter) = self.level_filter.read() else {
            return false;
        };

        if *filter != log::LevelFilter::Trace
            && NOISY_TARGET_PREFIXES
                .iter()
                .any(|prefix| metadata.target().starts_with(prefix))
        {
            return metadata.level() <= log::LevelFilter::Warn;
        }

        metadata.level() <= *filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Format the log message
        let level = record.level();
        let target = record.target();
        let message = format!("{}", record.args());

        // Skip empty messages
        if message.trim().is_empty() {
            return;
        }

        // Format the complete log line
        let log_line = format_log(level, target, &message);

        // Extract level string for append_log
        let level_str = match level {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };

        // Append to global log buffer with level filtering
        crate::dure_sijang_app::append_log(level_str, log_line.clone());

        // Print to stdout/stderr
        match level {
            log::Level::Error => eprintln!("{}", log_line),
            _ => println!("{}", log_line),
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<CombinedLogger> = OnceLock::new();

/// Initialize the combined logger that writes to both logcat and in-app log capture
pub fn init_combined_logger(level_filter: log::LevelFilter) {
    let logger = LOGGER.get_or_init(|| CombinedLogger {
        level_filter: Arc::new(RwLock::new(level_filter)),
    });

    // Set as the global logger
    if log::set_logger(logger).is_ok() {
        log::set_max_level(level_filter);
    }

    // On Android, also initialize android_logger as a backup
    #[cfg(target_os = "android")]
    {
        let _ = android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(level_filter)
                .with_tag("Dure-Sijang"),
        );
    }
}

/// Update the log level at runtime
pub fn update_log_level(level: &str) {
    let level_filter = match level.to_uppercase().as_str() {
        "TRACE" => log::LevelFilter::Trace,
        "DEBUG" => log::LevelFilter::Debug,
        "INFO" => log::LevelFilter::Info,
        "WARN" => log::LevelFilter::Warn,
        "ERROR" => log::LevelFilter::Error,
        _ => log::LevelFilter::Error,
    };

    if let Some(logger) = LOGGER.get() {
        if let Ok(mut filter) = logger.level_filter.write() {
            *filter = level_filter;
            log::set_max_level(level_filter);
        }
    }
}
