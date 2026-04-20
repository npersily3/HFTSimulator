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
use crossbeam::channel::{unbounded, Receiver, RecvError, Sender};

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
enum Messages {
    Tick,
    End,
    Start
}
pub struct {

}

pub struct  TickBarrier {
    sender: Sender<Messages>,
    receiver: Receiver<Messages>,
    pub(crate) wait_counter: AtomicUsize
}

impl TickBarrier {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        let tick_barrier = TickBarrier {
            sender,
            receiver,
            wait_counter: AtomicUsize::new(0)
        };
        tick_barrier
    }

    pub fn wait(&self) -> Messages {
        let counter = self.wait_counter.fetch_add(1, Ordering::Relaxed) + 1;
        if(counter == NUM_TRADER_THREADS) {
            self.wait_counter.store(0, Ordering::Relaxed);
            for i in 0..(NUM_TRADER_THREADS - 1)  {
                self.sender.send(Messages::Tick);
            }

            return Messages::Tick;

        }

        let result = self.receiver.recv();
        match result {
            Ok(_) => {
                result.unwrap()
            }
            Err(_) => {
                panic!("channel disconnected")
            }
        }
    }
    pub fn signal_end(&self) {
        for i in 0..NUM_TRADER_THREADS {
            self.sender.send(Messages::End).unwrap();
        }

    }
}
