use disco::{DiscoLogger, LoggerConfig, RecordFormat};
use log::LevelFilter;

fn main() {
    // setup logger
    let config = LoggerConfig {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Json,
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
