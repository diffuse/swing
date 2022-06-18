use disco::{ColorFormat, Config, DiscoLogger, RecordFormat};
use log::LevelFilter;
mod util;

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Simple,
        color_format: Some(ColorFormat::InlineGradient),
        ..Default::default()
    };
    DiscoLogger::new(config).init().unwrap();

    util::log_sample_messages(10);
}
