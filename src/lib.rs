#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://i.imgur.com/xOjqfvy.gif",
    html_favicon_url = "https://i.imgur.com/Q7UUHoN.png",
    html_playground_url = "https://play.rust-lang.org/"
)]
#![deny(missing_docs)]

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
mod paint;
use paint::LogPainter;
mod sculpt;
use sculpt::LogSculptor;
mod write;
use write::LogWriter;

pub mod color;
pub mod theme;
pub use color::{Rgb, RgbRange};
pub use paint::ColorFormat;
pub use sculpt::RecordFormat;
pub mod config;
pub use config::Config;
use theme::Theme;

/// Implements log::Log
pub struct DiscoLogger {
    /// Level-based filter for logs
    level_filter: LevelFilter,
    /// painter for logs
    log_painter: LogPainter,
    /// sculptor for logs
    log_sculptor: LogSculptor,
    /// writer for logs
    log_writer: LogWriter,
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
            log_sculptor: LogSculptor::new(config.record_format),
            level_filter: config.level,
            log_painter: LogPainter::new(config.theme, config.color_format),
            log_writer: LogWriter::new(config.use_stderr),
        }
    }

    /// Initialize this logger
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Trace))
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
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut msg = self.log_sculptor.sculpt(record);
        msg = self.log_painter.paint(msg, record.level());
        self.log_writer.write(msg, record.level());
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

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
