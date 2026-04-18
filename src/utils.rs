// use crossbeam::channel::{Sender, Receiver, bounded, unbounded};
// use std::sync::{Arc, OnceLock,LazyLock};
//
// static SYSTEM_SHUTDOWN: OnceLock<Sender<()>> = OnceLock::new();
//
//
// struct Event {
//
//
//
// }
// fn init_event(sender: Sender<()>) {
//     let (tx, rx) = bounded(0);
//
//     sender.
// }
//
// fn set_event(sender: Sender<()>) {
//     //SYSTEM_SHUTDOWN
// }

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Barrier;

pub static SYSTEM_END: AtomicBool = AtomicBool::new(false);
#[cfg(debug_assertions)]
macro_rules! ASSERT {
    ($x:expr) => {
        if !($x) {
            unsafe {
                DebugBreak();
            }
        }
    };
}
#[cfg(not(debug_assertions))]
macro_rules! ASSERT {
    ($x:expr) => {
        return;
    };
}
pub fn init() {
    std::panic::set_hook(Box::new(|info| {
        println!("Panic: {info}");
        unsafe { core::arch::asm!("int3"); }
    }));
}

pub(crate) use ASSERT;
use crate::NUM_TRADER_THREADS;

#[cfg(not(debug_assertions))]
macro_rules! ASSERT {
    ($x:expr) => {};
}

pub struct  TickBarrier {
    barrier: Barrier,
    pub(crate) wait_counter: AtomicUsize
}

impl TickBarrier {
    pub fn new(thread_count: usize) -> Self {
        let tick_barrier = TickBarrier {
            barrier: Barrier::new(thread_count),
            wait_counter: AtomicUsize::new(0)
        };
        tick_barrier
    }

    pub fn wait(&self) {
        let counter = self.wait_counter.fetch_add(1, Ordering::Relaxed);

        if(counter == NUM_TRADER_THREADS) {
            self.wait_counter.store(0, Ordering::Relaxed);
        }

        self.barrier.wait();
    }
}
