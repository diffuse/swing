use chrono::prelude::*;
use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;

/// Record formatting mode
pub enum RecordFormat {
    Json,
    Simple,
    Custom(Box<dyn Sync + Send + Fn(&Record) -> String>),
}

/// Config for logger
pub struct LoggerConfig {
    /// log level
    pub level: LevelFilter,
    /// record formatting mode
    pub record_format: RecordFormat,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            level: LevelFilter::Info,
            record_format: RecordFormat::Json,
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

/// Format a log message based on the current RecordFormat setting
fn format_record(record_format: &RecordFormat, record: &Record) -> String {
    let now = Utc::now().to_rfc3339();

    match record_format {
        RecordFormat::Json => json!({
            "time": now,
            "level": record.level(),
            "target": record.target(),
            "message": record.args(),
        })
        .to_string(),
        RecordFormat::Simple => {
            format!(
                "{} [{}] {} - {}",
                now,
                record.target(),
                record.level(),
                record.args()
            )
        }
        RecordFormat::Custom(f) => f(record),
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
            let mut msg = format_record(&self.config.record_format, record);
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
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn format_record_presets_return_non_empty() {
        // create normal test record
        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        // record should give non-empty log line
        assert!(!format_record(&RecordFormat::Json, &rec).is_empty());
        assert!(!format_record(&RecordFormat::Simple, &rec).is_empty());

        // create record with empty args and target
        let rec = Record::builder()
            .args(format_args!(""))
            .level(Level::Info)
            .target("")
            .build();

        // record should still give non-empty log lines
        assert!(!format_record(&RecordFormat::Json, &rec).is_empty());
        assert!(!format_record(&RecordFormat::Simple, &rec).is_empty());
    }

    #[test]
    fn format_record_custom_formats_correctly() {
        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        assert_eq!(
            format_record(&RecordFormat::Custom(Box::new(|_| "".to_string())), &rec),
            ""
        );

        assert_eq!(
            format_record(
                &RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args()))),
                &rec
            ),
            "INFO foo"
        );

        assert_eq!(
            format_record(
                &RecordFormat::Custom(Box::new(|r| format!(
                    "{} [{}] {}",
                    r.level(),
                    r.target(),
                    r.args()
                ))),
                &rec
            ),
            "INFO [test] foo"
        );
    }
}
