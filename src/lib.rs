#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://i.imgur.com/xOjqfvy.gif",
    html_favicon_url = "https://i.imgur.com/Q7UUHoN.png",
    html_playground_url = "https://play.rust-lang.org/"
)]
#![deny(missing_docs)]

use colored::Colorize;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;
use std::io;
use std::io::Write;
use std::sync::Mutex;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;
mod paint;
use paint::LogPainter;

pub mod color;
pub mod theme;
pub use color::{Rgb, RgbRange};
pub use paint::ColorFormat;
use theme::Theme;

/// Record formatting mode that determines how log records are structured
pub enum RecordFormat {
    /// JSON format
    Json,
    /// simple log format `<timestamp> [<target>] - <message>`
    Simple,
    /// custom record formatter provided by client code
    Custom(Box<dyn Sync + Send + Fn(&Record) -> String>),
}

/// Main configuration for a `DiscoLogger`
pub struct Config {
    /// log level filter (logs below this severity will be ignored)
    pub level: LevelFilter,
    /// record formatting mode (determines how log records are structurally formatted)
    pub record_format: RecordFormat,
    /// color formatting mode (determines how log records are colored)
    pub color_format: Option<ColorFormat>,
    /// color theme (determines the color palette used to color log records)
    pub theme: Box<dyn Theme>,
    /// switch for enabling log splitting to `stderr`
    ///
    /// - `true`: log `trace` - `info` levels to `stdout` and `warn` - `error` levels to `stderr`
    ///
    /// - `false`: log all levels to `stdout`
    pub use_stderr: bool,
}

impl Default for Config {
    /// Return a `Config` with default values
    fn default() -> Config {
        Config {
            level: LevelFilter::Info,
            record_format: RecordFormat::Simple,
            color_format: Some(ColorFormat::Solid),
            theme: Box::new(theme::Simple {}),
            use_stderr: true,
        }
    }
}

/// Implements log::Log
pub struct DiscoLogger {
    /// Record format for logs
    record_format: RecordFormat,
    /// Level-based filter for logs
    level_filter: LevelFilter,
    /// switch for enabling log splitting to `stderr`
    ///
    /// - `true`: log `trace` - `info` levels to `stdout` and `warn` - `error` levels to `stderr`
    ///
    /// - `false`: log all levels to `stdout`
    use_stderr: bool,
    /// painter for logs
    log_painter: LogPainter,
    /// guard against interleaving from simultaneous writes to stdout + stderr
    write_mtx: Mutex<()>,
    /// handle to stdout
    stdout: io::Stdout,
    /// handle to stderr
    stderr: io::Stderr,
}

impl DiscoLogger {
    /// Create a new DiscoLogger with a default configuration
    pub fn new() -> DiscoLogger {
        DiscoLogger::with_config(Config::default())
    }

    /// Create a new DiscoLogger with a custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - configuration for this logger
    pub fn with_config(config: Config) -> DiscoLogger {
        DiscoLogger {
            record_format: config.record_format,
            level_filter: config.level,
            log_painter: LogPainter::new(config.theme, config.color_format),
            use_stderr: config.use_stderr,
            write_mtx: Mutex::new(()),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }

    /// Initialize this logger
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Trace))
    }

    /// Convert a log record into a formatted string, based on the current logger configuration
    ///
    /// # Arguments
    ///
    /// * `record` - the log record to format
    fn format_record(&self, record: &Record) -> String {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .expect("Failed to format time as ISO 8601");

        match &self.record_format {
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
}

impl Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    /// Log a message/record
    ///
    /// # Arguments
    ///
    /// * `record` - the record to log
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = self.format_record(record);
            msg = self.log_painter.paint(msg, record.level());

            // stdout and stderr already have their own locks, but
            // there is nothing preventing logs simultaneously written
            // to stdout + stderr from being interleaved in the console
            //
            // this guard synchronizes writes so that stdout will not be
            // interleaved with stderr
            let _lk = self.write_mtx.lock().unwrap();

            match record.level() {
                Level::Warn | Level::Error => {
                    if self.use_stderr {
                        let _ = writeln!(self.stderr.lock(), "{}", msg.bold());
                    } else {
                        let _ = writeln!(self.stdout.lock(), "{}", msg.bold());
                    }
                }
                _ => {
                    let _ = writeln!(self.stdout.lock(), "{}", msg);
                }
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_record_presets_return_non_empty() {
        for fmt in vec![RecordFormat::Json, RecordFormat::Simple] {
            let config = Config {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::with_config(config);

            // create normal test record
            let rec = Record::builder()
                .args(format_args!("foo"))
                .level(Level::Info)
                .target("test")
                .build();

            assert!(!logger.format_record(&rec).is_empty());

            // create record with empty args and target
            let rec = Record::builder()
                .args(format_args!(""))
                .level(Level::Info)
                .target("")
                .build();

            // record should still give non-empty log lines
            assert!(!logger.format_record(&rec).is_empty());
        }
    }

    #[test]
    fn format_record_custom_formats_correctly() {
        let test_cases = vec![
            (RecordFormat::Custom(Box::new(|_| "".to_string())), ""),
            (
                RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args()))),
                "INFO foo",
            ),
            (
                RecordFormat::Custom(Box::new(|r| {
                    format!("{} [{}] {}", r.level(), r.target(), r.args())
                })),
                "INFO [test] foo",
            ),
        ];

        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        for (fmt, expected) in test_cases {
            let config = Config {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::with_config(config);

            assert_eq!(logger.format_record(&rec), expected);
        }
    }

    #[test]
    fn enabled_filters_levels() {
        let config = Config {
            level: LevelFilter::Warn,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn enabled_with_off_level_filter_is_always_false() {
        let config = Config {
            level: LevelFilter::Off,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn log_handles_empty_record() {
        let config = Config::default();
        let logger = DiscoLogger::with_config(config);

        // create record with fields set to empty strings
        let rec = Record::builder()
            .args(format_args!(""))
            .level(Level::Info)
            .target("")
            .build();

        // this shouldn't panic
        logger.log(&rec);
    }
}
