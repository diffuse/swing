use log::LevelFilter;
use swing::{theme::Spectral, ColorFormat, Config, Logger, RecordFormat};
mod util;

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Simple,
        color_format: Some(ColorFormat::InlineGradient(200)),
        theme: Box::new(Spectral {}),
        ..Default::default()
    };
    Logger::with_config(config).init().unwrap();

    util::log_sample_messages(100);
}
