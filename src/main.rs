use crate::exchange::{Book, MarketOrder};
use crate::market::INITIAL_MONEY;
use crossbeam::channel::{Receiver, SendError, Sender, bounded, unbounded};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

mod exchange;
mod market;
mod utils;
mod corporation;

fn seeder(sender: Sender<MarketOrder>, initial_true_value: u64) {
    let money = Arc::new(AtomicU64::new(INITIAL_MONEY));
    let is_canceled = Arc::new(AtomicBool::new(false));

    // seed before waiting on barrier
    for i in 1..=5 {
        exchange::limit_ask(
            initial_true_value + i,
            is_canceled.clone(),
            100,
            money.clone(),
            sender.clone(),
        )
        .unwrap();
        exchange::limit_bid(
            initial_true_value - i,
            is_canceled.clone(),
            100,
            money.clone(),
            sender.clone(),
        )
        .unwrap();
    }
}

const NUM_NOISE_THREADS: usize = 5;
const NUM_FUNDAMENTAL_THREADS: usize = 5;
const NUM_EXCHANGE_THREADS: usize = 1;
const NUM_PERSONAL_THREADS: usize = 0;
fn main() {
    // later set to bounded and compare
    let (sender, receiver) = unbounded();
    let start = Arc::new(Barrier::new(
        NUM_FUNDAMENTAL_THREADS + NUM_NOISE_THREADS + NUM_EXCHANGE_THREADS + NUM_PERSONAL_THREADS,
    ));
    let tick = Arc::new(Barrier::new(NUM_FUNDAMENTAL_THREADS + NUM_NOISE_THREADS));
    let bid_index = Arc::new(AtomicUsize::new(0));
    let ask_index = Arc::new(AtomicUsize::new(0));
    let true_price = Arc::new(AtomicU64::new(0));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for _ in 0..NUM_FUNDAMENTAL_THREADS {
        // if have to do it this way because the move transfers ownership
        let sender = sender.clone();
        let start = start.clone();
        let tick = tick.clone();
        let bid_index = bid_index.clone();
        let ask_index = ask_index.clone();
        let true_price = true_price.clone();

       let handle = std::thread::spawn(move || {
            market::fundamentalist(
                sender,
                start,
                Option::from(tick),
                bid_index,
                ask_index,
                true_price,
            );
        });

        handles.push(handle);
    }

    for _ in 0..NUM_NOISE_THREADS {
        // if have to do it this way because the move transfers ownership
        let sender = sender.clone();
        let start = start.clone();
        let tick = tick.clone();


        let handle = std::thread::spawn(move || {
            market::noise(sender, start, Option::from(tick));
        });
        handles.push(handle);
    }

    let mut order_book = Book::new(bid_index.clone(), ask_index.clone());
    seeder(sender.clone(), 1000);
    //for _ in 0..NUM_EXCHANGE_THREADS {
    let receiver = receiver.clone();
    let tick = tick.clone();

   let handle= std::thread::spawn(move || {
        exchange::handle_orders(receiver, &mut order_book, tick);
    });

    handles.push(handle);

    sleep(Duration::from_secs(1));
    utils::SYSTEM_END.store(true, Ordering::Relaxed);
    println!("finished");
    for handle in handles {
        handle.join().unwrap();
    }


}
