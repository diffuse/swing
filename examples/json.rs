use disco::{Config, DiscoLogger, RecordFormat};
use log::LevelFilter;

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Json,
        ..Default::default()
    };
    DiscoLogger::with_config(config).init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
