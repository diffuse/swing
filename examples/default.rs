use swing::Logger;

fn main() {
    // setup logger
    Logger::new().init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
