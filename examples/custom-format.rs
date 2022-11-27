use log::{LevelFilter, Record};
use swing::{Config, Logger, RecordFormat};

fn main() {
    // create your own conversion from a
    // log::Record to a String and wrap it
    // in a Box to use with a Logger
    let fmt_rec = Box::new(|r: &Record| -> String {
        format!("[{}] {} - {}", r.target(), r.level(), r.args())
    });

    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Custom(fmt_rec),
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
