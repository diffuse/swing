use lipsum::lipsum;
use rand::Rng;

/// log n sample messages, weighted towards the default _ match arm
pub fn log_sample_messages(n: usize) {
    for _ in 0..n {
        let n = rand::thread_rng().gen_range(0..20);
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

#[allow(dead_code)]
fn main() {}
