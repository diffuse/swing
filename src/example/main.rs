use disco::{DiscoLogger, LoggerConfig, LoggerMode};
use log::LevelFilter;

fn main() {
    // setup logger
    let config = LoggerConfig {
        level: LevelFilter::Trace,
        mode: LoggerMode::Json,
        ..Default::default()
    };
    DiscoLogger::new(config).init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
