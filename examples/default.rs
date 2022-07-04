use disco::DiscoLogger;

fn main() {
    // setup logger
    DiscoLogger::new().init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
