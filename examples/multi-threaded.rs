use swing::{theme::Spectral, ColorFormat, Config, SwingLogger};
use log::LevelFilter;
use std::thread;
mod util;

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        color_format: Some(ColorFormat::MultiLineGradient(20)),
        theme: Box::new(Spectral {}),
        ..Default::default()
    };
    SwingLogger::with_config(config).init().unwrap();

    // log sample messages from 10 threads simultaneously
    let mut handles = vec![];

    for _ in 0..10 {
        handles.push(thread::spawn(move || {
            util::log_sample_messages(1000);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
