use chrono::prelude::*;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use serde_json::json;

/// Config for logger
pub struct LoggerConfig {
    /// log level
    pub level: LevelFilter,
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

impl log::Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    /// Log a message
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = json!({
                "time": Utc::now().to_rfc3339(),
                "level": record.level(),
                "target": record.target(),
                "message": record.args(),
            });

            match record.level() {
                Level::Warn | Level::Error => {
                    eprintln!("{}", msg)
                }
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
    fn it_works() {
        /*
        let config = LoggerConfig {
            level: LevelFilter::Info,
        };
        DiscoLogger::new(config).init().unwrap();
        log::info!("foo");
        */
    }
}
