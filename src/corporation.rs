use crate::utils;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use crate::utils::TickBarrier;

pub const STARTING_PRICE: u64 = 1000;
pub fn set_true_price(true_price: Arc<AtomicU64>, start: Arc<Barrier>, tick: Arc<TickBarrier>) {
    start.wait();
    println!("started");
    loop {
        if utils::SYSTEM_END.load(Ordering::Relaxed) {
            break;
        }

        match rand::random::<bool>() {
            true => {
                true_price.fetch_sub(1, Ordering::Relaxed);
            }
            false => {
                true_price.fetch_add(1, Ordering::Relaxed);
            }
        }

        //TODO add a mode where it is not tick based
        tick.wait();
    }
}
