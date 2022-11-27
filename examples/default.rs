use swing::SwingLogger;

fn main() {
    // setup logger
    SwingLogger::new().init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
