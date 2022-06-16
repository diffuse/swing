use disco::theme::Spectral;
use disco::{ColorFormat, DiscoLogger, LoggerConfig, RecordFormat};
use log::LevelFilter;
mod util;

fn main() {
    // setup logger
    let config = LoggerConfig {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Simple,
        color_format: Some(ColorFormat::MultiLineGradient),
        theme: Box::new(Spectral {}),
        ..Default::default()
    };
    DiscoLogger::new(config).init().unwrap();

    util::log_sample_messages(1000);
}
