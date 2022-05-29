use chrono::prelude::*;
use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;

/// Logger formatting mode
pub enum LoggerMode {
    Json,
    Simple,
}

/// Config for logger
pub struct LoggerConfig {
    /// log level
    pub level: LevelFilter,
    /// logger mode
    pub mode: LoggerMode,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            level: LevelFilter::Info,
            mode: LoggerMode::Json,
        }
    }
}

/// Implements log::Log
pub struct DiscoLogger {
    config: LoggerConfig,
}

impl DiscoLogger {
    pub fn new(config: LoggerConfig) -> DiscoLogger {
        DiscoLogger { config }
    }

    /// Initialize this logger
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Trace))
    }
}

/// Color code and bold level strings as appropriate
fn format_by_level(level: Level, msg: String) -> ColoredString {
    match level {
        Level::Trace => msg.bright_magenta(),
        Level::Debug => msg.cyan(),
        Level::Info => msg.green(),
        Level::Warn => msg.bold().truecolor(255, 128, 0),
        _ => msg.bold().red(),
    }
}

/// Format a log message based on the current LoggerMode setting
fn format_by_mode(mode: &LoggerMode, record: &Record) -> String {
    let now = Utc::now().to_rfc3339();

    match mode {
        LoggerMode::Json => json!({
            "time": now,
            "level": record.level(),
            "target": record.target(),
            "message": record.args(),
        })
        .to_string(),
        LoggerMode::Simple => {
            format!(
                "{} [{}] {} - {}",
                now,
                record.target(),
                record.level(),
                record.args()
            )
        }
    }
}

impl Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    /// Log a message
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = format_by_mode(&self.config.mode, record);
            msg = format_by_level(record.level(), msg).to_string();

            match record.level() {
                Level::Warn | Level::Error => eprintln!("{}", msg),
                _ => println!("{}", msg),
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_filters_levels() {
        let config = LoggerConfig {
            level: LevelFilter::Warn,
        };
        let logger = DiscoLogger::new(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(logger.enabled(&mut mb.level(Level::Error).build()));
    }
}
