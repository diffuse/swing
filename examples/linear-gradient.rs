use disco::{ColorFormat, DiscoLogger, LoggerConfig, RecordFormat};
use lipsum::lipsum;
use log::LevelFilter;
use rand::Rng;

/// log n sample messages, weighted towards the default _ match arm
fn log_sample_messages(n: usize) {
    for _ in 0..n {
        let n = rand::thread_rng().gen_range(0..50);
        let msg = lipsum(n);

        match rand::thread_rng().gen_range(0..15) {
            0 => log::trace!("{}", msg),
            1 => log::debug!("{}", msg),
            2 => log::info!("{}", msg),
            3 => log::warn!("{}", msg),
            4 => log::error!("{}", msg),
            _ => log::info!("{}", msg),
        };
    }
}

fn main() {
    // setup logger
    let config = LoggerConfig {
        level: LevelFilter::Trace,
        record_format: RecordFormat::Simple,
        color_format: Some(ColorFormat::LinearGradient),
        ..Default::default()
    };
    DiscoLogger::new(config).init().unwrap();

    log_sample_messages(10);
}
