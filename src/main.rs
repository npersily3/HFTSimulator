use crate::exchange::{Book, HistoryEntry, MarketOrder};
use crate::market::INITIAL_MONEY;
use crossbeam::channel::{Sender, unbounded};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use crossbeam::queue::ArrayQueue;
use windows_sys::Win32::System::Diagnostics::Debug::DebugBreak;
use crate::utils::TickBarrier;

mod corporation;
mod exchange;
mod market;
mod utils;




const NUM_NOISE_THREADS: usize = 3;
const NUM_FUNDAMENTAL_THREADS: usize = 3;
const NUM_EXCHANGE_THREADS: usize = 1;
const NUM_CORPORATATION_THREADS: usize = 1;
const NUM_PERSONAL_THREADS: usize = 0;

const NUM_TRADER_THREADS: usize = NUM_PERSONAL_THREADS+ NUM_FUNDAMENTAL_THREADS + NUM_NOISE_THREADS ;
const NUM_THREADS: usize = NUM_FUNDAMENTAL_THREADS
    + NUM_NOISE_THREADS
    + NUM_EXCHANGE_THREADS
    + NUM_PERSONAL_THREADS
    + NUM_CORPORATATION_THREADS;

const HISTORY_SIZE: usize = 1 << 15;
fn main() {
    utils::init();
    // later set to bounded and compare
    let (sender, receiver) = unbounded();
    let start = Arc::new(Barrier::new(
        NUM_THREADS,
    ));
    let tick = Arc::new(TickBarrier::new(
        NUM_FUNDAMENTAL_THREADS + NUM_NOISE_THREADS + NUM_CORPORATATION_THREADS,
    ));
    let highest_bid_index = Arc::new(AtomicUsize::new(0));
    let lowest_ask_index = Arc::new(AtomicUsize::new(0));
    let true_price = Arc::new(AtomicU64::new(corporation::STARTING_PRICE));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for i in 0..NUM_FUNDAMENTAL_THREADS {
        // if have to do it this way because the move transfers ownership
        let sender = sender.clone();
        let start = start.clone();
        let tick = tick.clone();
        let bid_index = highest_bid_index.clone();
        let ask_index = lowest_ask_index.clone();
        let true_price = true_price.clone();

        let handle = std::thread::Builder::new()
            .name(format!("fundamentalist {}", i))
            .spawn(move || {

                market::fundamentalist(
                    sender,
                    start,
                    Option::from(tick),
                    bid_index,
                    ask_index,
                    true_price,
                );
            })
            .unwrap();

        handles.push(handle);
    }


    for i in 0..NUM_NOISE_THREADS {
        // if have to do it this way because the move transfers ownership
        let sender = sender.clone();
        let start = start.clone();
        let tick = tick.clone();

        let handle = std::thread::Builder::new()
            .name(format!("noise_{}", i))
            .spawn(move || {
                market::noise(sender, start, Option::from(tick));
            })
            .unwrap();
        handles.push(handle);
    }


    let highest_ask_index = Arc::new(AtomicUsize::new(0));
    let lowest_bid_index = Arc::new(AtomicUsize::new(0));


    let mut order_book = Book::new(highest_ask_index.clone(),lowest_ask_index.clone(),highest_bid_index.clone(),lowest_bid_index.clone());
    let order_history: ArrayQueue<HistoryEntry> = ArrayQueue::new(HISTORY_SIZE);

    //for _ in 0..NUM_EXCHANGE_THREADS {
    let receiver = receiver.clone();
    let tick1 = tick.clone();
    let start1 = start.clone();
    let handle = std::thread::Builder::new()
        .name("exchange".to_string())
        .spawn(move || {
            exchange::handle_orders(receiver, &mut order_book, tick1,start1, &order_history);
        })
        .unwrap();

    handles.push(handle);

    let tick = tick.clone();
    let true_price = true_price.clone();
    let start = start.clone();

    let handle = std::thread::Builder::new()
        .name("corporation".to_string())
        .spawn(move || {
            corporation::set_true_price(true_price, start, tick);
        })
        .unwrap();

    handles.push(handle);

    sleep(Duration::from_secs(5));
    utils::SYSTEM_END.store(true, Ordering::Relaxed);

    for handle in handles {
        handle.join().unwrap();
    }
}

//markov processes, discrete and continouts, poisson (random times between one event and other event), birth death process, monte carlo process
