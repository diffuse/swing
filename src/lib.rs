use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/// Config for logger
pub struct LoggerConfig {
    /// log level
    level: LevelFilter,
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
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Info))
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
            let msg = format!("{} - {}", record.level(), record.args());

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
    #[test]
    fn it_works() {
        /*
        let config = LoggerConfig {
            level: LevelFilter::Info,
        };
        DiscoLogger::new(config).init().unwrap();
        log::info!("");
        */
    }
}
