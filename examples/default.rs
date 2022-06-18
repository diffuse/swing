use disco::{Config, DiscoLogger};

fn main() {
    // setup logger
    let config = Config::default();
    DiscoLogger::new(config).init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
