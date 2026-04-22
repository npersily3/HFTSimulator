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

use crossbeam::channel::{Receiver, RecvError, Sender, unbounded};
use crossbeam::select;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Barrier, Condvar, Mutex};

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
      //  println!("Panic: {info}");
        unsafe {
            core::arch::asm!("int3");
        }
    }));
}

use crate::NUM_TRADER_THREADS;
pub(crate) use ASSERT;

#[cfg(not(debug_assertions))]
macro_rules! ASSERT {
    ($x:expr) => {};
}

pub struct TickBarrier {
    pub(crate) cvar: Condvar,
    lock: Mutex<usize>,

    pub(crate) wait_counter: AtomicUsize,
}

impl TickBarrier {
    pub fn new() -> Self {
        let tick_barrier = TickBarrier {
            cvar: Condvar::new(),
            lock: Mutex::new(0),
            wait_counter: AtomicUsize::new(0),
        };
        tick_barrier
    }

    pub fn wait(&self) {
        let mut guard = self.lock.lock().unwrap();
        self.wait_counter.fetch_add(1, Ordering::SeqCst);
        let my_age = *guard;

        // wait while terminates when false so when the age is advance
        let mut guard = self
            .cvar
            .wait_while(guard, |wakeup| {
                *wakeup == my_age && !SYSTEM_END.load(Ordering::Relaxed)
            })
            .unwrap();
    }
    pub fn wake(&self) {
        let mut guard = self.lock.lock().unwrap();
        self.wait_counter.store(0, Ordering::SeqCst);
        *guard += 1;
        self.cvar.notify_all();
    }

    pub fn shutdown(&self) {
        let mut guard = self.lock.lock().unwrap();
        SYSTEM_END.store(true, Ordering::Relaxed);
        self.cvar.notify_all();
    }
}
