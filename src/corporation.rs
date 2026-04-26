use crate::utils;
use crate::utils::TickBarrier;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};


pub const STARTING_PRICE: u64 = 1000;
const RANDOM_WALK: u64 = 20;
pub fn set_true_price(true_price: Arc<AtomicU64>, start: Arc<Barrier>, tick: Arc<TickBarrier>) {
    start.wait();
    //println!("started");
    loop {
        if utils::SYSTEM_END.load(Ordering::Relaxed) {
            break;
        }

        match rand::random::<bool>() {
            true => {
                true_price.fetch_sub(RANDOM_WALK, Ordering::Relaxed);
            }
            false => {
                true_price.fetch_add(RANDOM_WALK, Ordering::Relaxed);
            }
        }

        #[cfg(feature = "tick")]
        {
            tick.wait();
        }
        #[cfg(feature = "time")]
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
