use crate::{DebugBreak, NUM_TRADER_THREADS};
use crate::utils;
use crate::utils::{TickBarrier, ASSERT};
use crate::corporation;
use crossbeam::channel::{Receiver, SendError, Sender};
use crossbeam::queue::ArrayQueue;
use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, LazyLock};
use std::time::Instant;
use std::io::Write;
// A file containing the implementation

#[derive(Debug)]
pub enum OrderType {
    Ask,
    Bid,
}
pub struct MarketOrder {
    order_type: OrderType,
    price: u64,
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
}
impl fmt::Display for MarketOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MarketOrder {{ type: {:?}, price: {}, quantity: {}, canceled: {}, money: {} }}",
            self.order_type,
            self.price,
            self.quantity,
            self.is_canceled.load(Ordering::Relaxed),
            self.money_address.load(Ordering::Relaxed),
        )
    }
}
pub struct HistoryEntry {
    order_type: OrderType,
    price: u64,
    quantity: u32,
    timestamp: Instant,
}
const HISTORY_SIZE: usize = 1 << 15;


struct QueueEntry {
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
}
struct BookEntry {
    bids: VecDeque<QueueEntry>,
    asks: VecDeque<QueueEntry>,
}

impl BookEntry {
    fn new() -> Self {
        let bids: VecDeque<QueueEntry> = VecDeque::new();
        let asks: VecDeque<QueueEntry> = VecDeque::new();

        let book_entry = BookEntry { bids, asks };
        book_entry
    }
}

const BOOK_SIZE: usize = 1 << 15;
pub struct Book {
    // you are accepting a race condition here because the user can update spread inbetween calculations
    highest_ask_index: Arc<AtomicUsize>,
    lowest_ask_index: Arc<AtomicUsize>,

    highest_bid_index: Arc<AtomicUsize>,
    lowest_bid_index: Arc<AtomicUsize>,
    table: Vec<BookEntry>,
}
impl Book {
    //TODO
    pub(crate) fn new(highest_ask_index: Arc<AtomicUsize>, lowest_ask_index: Arc<AtomicUsize>, highest_bid_index: Arc<AtomicUsize>, lowest_bid_index: Arc<AtomicUsize>) -> Book {
        let table: Vec<BookEntry> = (0..BOOK_SIZE).map(|_| BookEntry::new()).collect();

        let book = Book {
            highest_ask_index,
            lowest_ask_index,
            highest_bid_index,
            lowest_bid_index,
            table,
        };
        book
    }
}

pub fn limit_ask(
    price: u64,
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
    sender: Sender<MarketOrder>,
) -> Result<(), SendError<MarketOrder>> {
    if is_canceled.load(Ordering::Relaxed) {
        //figure out how to return safely;
        panic!("stop trying to f up the exchange")
    }

    let order = MarketOrder {
        order_type: OrderType::Ask,
        price,
        is_canceled,
        quantity,
        money_address,
    };

    sender.send(order)
}
pub fn limit_bid(
    price: u64,
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
    sender: Sender<MarketOrder>,
) -> Result<(), SendError<MarketOrder>> {
    if is_canceled.load(Ordering::Relaxed) {
        //figure out how to return safely;
        panic!("stop trying to f up the exchange")
    }

    let order = MarketOrder {
        order_type: OrderType::Bid,
        price,
        is_canceled,
        quantity,
        money_address,
    };

    sender.send(order)
}

//for non limit orders
pub fn ask(
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
    sender: Sender<MarketOrder>,
) -> Result<(), SendError<MarketOrder>> {
    limit_ask(0, is_canceled, quantity, money_address, sender)
}
pub fn bid(
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
    sender: Sender<MarketOrder>,
) -> Result<(), SendError<MarketOrder>> {
    limit_bid(0, is_canceled, quantity, money_address, sender)
}

fn handle_ask(market_order: MarketOrder, order_book: &mut Book, history_book: &ArrayQueue<HistoryEntry>) {
    let mut index = order_book.highest_bid_index.load(Ordering::Relaxed);


    //create local mutables of the order

    let mut price = 0;
    if market_order.price == 0 {
         price = order_book.lowest_bid_index.load(Ordering::Relaxed);
    } else {
         price = market_order.price as usize;
    }

    let mut ask_quantity = market_order.quantity;

    //TODO handle non limit orders better
    //traversing from the highest to lowest bid we are willing to go to
    while index >= price {
        //if we have any bids at the current price
        while order_book.table[index].bids.is_empty() == false {
            let current_bid = order_book.table[index].bids.front_mut();
            ASSERT!(current_bid.is_some());
            let current_bid = current_bid.unwrap();

            // if it was cancelled pop it off and move over
            if current_bid.is_canceled.load(Ordering::Relaxed) {
                order_book.table[index].bids.pop_front();
                continue;
            }

            let ask_is_smaller = current_bid.quantity > ask_quantity;
            let trade_quantity = if ask_is_smaller { ask_quantity } else { current_bid.quantity };
            let money_difference = trade_quantity as u64 * index as u64;

            if money_difference > current_bid.money_address.load(Ordering::Relaxed) {
                return;
            }

            if ask_is_smaller {
                current_bid.quantity -= ask_quantity;
            } else {
                ask_quantity -= current_bid.quantity;
            }

            market_order.money_address.fetch_add(money_difference, Ordering::Relaxed);
            current_bid.money_address.fetch_sub(money_difference, Ordering::Relaxed);

            let record = HistoryEntry {
                order_type: OrderType::Ask,
                price: money_difference,
                quantity: market_order.quantity,
                timestamp: Instant::now(),
            };

            // match history_book.push(record) {
            //     Ok(_) => {}
            //     Err(_) => {
            //         println!("history channel disconnected, program likely over");
            //         return;
            //     }
            // }

            if ask_is_smaller {
                order_book.table[index].bids.pop_front();
            }

            // if we have finished,  update the bid to the current one and then return
            if ask_quantity == 0 {
                order_book.lowest_bid_index.store(index, Ordering::Relaxed);
                return;
            }
        }

        index -= 1;
    }

    order_book
        .lowest_bid_index
        .store(price as usize, Ordering::Relaxed);

    let queue_entry = QueueEntry {
        is_canceled: market_order.is_canceled.clone(),
        quantity: ask_quantity,
        money_address: market_order.money_address.clone(),
    };
    // at this point there are no bids for what we want so we have to instantiate a new ask at our pricepoint
    order_book.table[price as usize].asks.push_back(queue_entry);

    // if the new ask will be the lowest ask
    if order_book.highest_ask_index.load(Ordering::Relaxed) > price as usize {
        order_book
            .highest_ask_index
            .store(price as usize, Ordering::Relaxed);
    }
}

fn handle_bid(market_order: MarketOrder, order_book: &mut Book, history_book: &ArrayQueue<HistoryEntry>) {
    let mut index = order_book.lowest_ask_index.load(Ordering::Relaxed);


    let mut price = 0;
    if market_order.price == 0 {
        price = order_book.highest_bid_index.load(Ordering::Relaxed);
    } else {
        price = market_order.price as usize;
    }

    //create local mutables of the order

    let mut bid_quantity = market_order.quantity;


    //traversing from the lowest to highest ask we are willing to go to
    while price >= index {
        //if we have any bids at the current price
        while order_book.table[index].asks.is_empty() == false {
            let current_ask = order_book.table[index].asks.front_mut();
            debug_assert!(current_ask.is_some());
            let current_ask = current_ask.unwrap();

            // if it was cancelled pop it off and move over
            if current_ask.is_canceled.load(Ordering::Relaxed) {
                order_book.table[index].asks.pop_front();
                continue;
            }

            let bid_is_smaller = current_ask.quantity > bid_quantity;
            let trade_quantity = if bid_is_smaller { bid_quantity } else { current_ask.quantity };
            let money_difference = trade_quantity as u64 * index as u64;

            if market_order.money_address.load(Ordering::Relaxed) < money_difference {
                return;
            }

            if bid_is_smaller {
                current_ask.quantity -= bid_quantity;
            } else {
                bid_quantity -= current_ask.quantity;
            }

            market_order.money_address.fetch_sub(money_difference, Ordering::Relaxed);
            current_ask.money_address.fetch_add(money_difference, Ordering::Relaxed);

            let record = HistoryEntry {
                order_type: OrderType::Bid,
                price: money_difference,
                quantity: market_order.quantity,
                timestamp: Instant::now(),
            };

            // match history_book.push(record) {
            //     Ok(_) => {}
            //     Err(_) => {
            //         println!("history channel disconnected, program likely over");
            //         return;
            //     }
            // }

            if bid_is_smaller {
                order_book.table[index].asks.pop_front();
            }

            if bid_quantity == 0 {
                order_book.highest_ask_index.store(index, Ordering::Relaxed);
                return;
            }
        }

        index += 1;
    }

    order_book
        .highest_ask_index
        .store(price as usize, Ordering::Relaxed);

    let queue_entry = QueueEntry {
        is_canceled: market_order.is_canceled.clone(),
        quantity: bid_quantity,
        money_address: market_order.money_address.clone(),
    };
    // at this point there are no bids for what we want so we have to instantiate a new ask at our pricepoint
    order_book.table[price as usize].bids.push_back(queue_entry);

    // if the new ask will be the lowest ask
    if order_book.lowest_bid_index.load(Ordering::Relaxed) < price as usize {
        order_book
            .lowest_bid_index
            .store(price as usize, Ordering::Relaxed);
    }
}

const INIT_RADIUS: usize = 50;
const INIT_QUANTITY: u32 = 200;
const INIT_BALANCE: u64 = 1_000_000_000;

fn init_exchange(book: &mut Book) {
    let mid = corporation::STARTING_PRICE as usize;

    // best ask = mid + 1 (lowest ask); best bid = mid - 1 (highest bid)
    book.highest_ask_index.store(mid + 1, Ordering::Relaxed);
    book.lowest_bid_index.store(mid - 1, Ordering::Relaxed);

    // outer edges of the seeded range
    book.lowest_ask_index.store(mid + INIT_RADIUS, Ordering::Relaxed);
    book.highest_bid_index.store(mid - INIT_RADIUS, Ordering::Relaxed);

    // seed asks from mid+1 up to mid+INIT_RADIUS
    for price in (mid + 1)..=(mid + INIT_RADIUS) {
        book.table[price].asks.push_back(QueueEntry {
            is_canceled: Arc::new(AtomicBool::new(false)),
            quantity: INIT_QUANTITY,
            money_address: Arc::new(AtomicU64::new(INIT_BALANCE)),
        });
    }

    // seed bids from mid-INIT_RADIUS up to mid-1
    for price in (mid - INIT_RADIUS)..=(mid - 1) {
        book.table[price].bids.push_back(QueueEntry {
            is_canceled: Arc::new(AtomicBool::new(false)),
            quantity: INIT_QUANTITY,
            money_address: Arc::new(AtomicU64::new(INIT_BALANCE)),
        });
    }
}

///pulls off queue and updates book should be its own thread
pub fn handle_orders(
    receiver: Receiver<MarketOrder>,
    mut order_book: &mut Book,
    tick: Arc<TickBarrier>,
    start: Arc<Barrier>,
    history_book: &ArrayQueue<HistoryEntry>,
    true_price: Arc<AtomicU64>,
) {
    init_exchange(&mut order_book);

    start.wait();

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut tick_num: u64 = 0;

    loop {
        loop {
            if utils::SYSTEM_END.load(Ordering::Relaxed) {
                return;
            }
            if tick.wait_counter.load(Ordering::SeqCst) >= NUM_TRADER_THREADS {
                break;
            }
        }

        let market_order = receiver.try_recv();

        match market_order {
            Ok(market_order) => {
                match market_order.order_type {
                    OrderType::Ask => {
                        handle_ask(market_order, &mut order_book, &history_book);
                    }
                    OrderType::Bid => {
                        handle_bid(market_order, &mut order_book, &history_book);
                    }
                }
            }
            Err(_) => {
                let best_bid = order_book.lowest_bid_index.load(Ordering::Relaxed);
                let best_ask = order_book.highest_ask_index.load(Ordering::Relaxed);
                let mid = (best_bid + best_ask) / 2;
                let true_p = true_price.load(Ordering::Relaxed);
                let _ = writeln!(out, "TICK:{},{},{},{},{}", tick_num, best_bid, best_ask, mid, true_p);
                let _ = out.flush();
                tick_num += 1;
                tick.wake();
            }
        }
    }
}
