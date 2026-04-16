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

use std::sync::atomic::AtomicBool;

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
#[cfg(not(debug_assertions))]
macro_rules! ASSERT {
    ($x:expr) => {};
}
