use log::LevelFilter;
use swing::{Config, Logger, RecordFormat};

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Json,
        ..Default::default()
    };
    Logger::with_config(config).init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
