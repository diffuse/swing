use disco::theme::Spectral;
use disco::{ColorFormat, Config, DiscoLogger, RecordFormat};
use log::LevelFilter;
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
    DiscoLogger::with_config(config).init().unwrap();

    util::log_sample_messages(100);
}
