use crossbeam::channel::{Receiver, SendError, Sender, bounded, unbounded};
use crossbeam::queue::ArrayQueue;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};

// A file containing the implementation

struct PricePair {
    price: u64,
    quantity: u32,
}

impl Default for PricePair {
    fn default() -> Self {
        PricePair {
            price: 0,
            quantity: 0,
        }
    }
}
//enum MarketOrder {
//
//     Ask { quantity: u32,
//         price: u64,
//         money_address: Arc<AtomicU64>
//     },
//
//     Bid {
//         quantity: u32,
//         money_address: Arc<AtomicU64> },
// }
enum OrderType {
    Ask,
    Bid,
}
struct MarketOrder {
    order_type: OrderType,
    price: u64,
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
}
struct HistoryEntry {
    market_order: MarketOrder,
    timestamp: u64,
}
const HISTORY_SIZE: usize = 1 << 15;
static MARKET_HISTORY: LazyLock<ArrayQueue<HistoryEntry>> =
    LazyLock::new(|| ArrayQueue::new(HISTORY_SIZE));

struct BookEntry {
    bids: VecDeque<MarketOrder>,
    asks: VecDeque<MarketOrder>,
}

const BOOK_SIZE: usize = 1 << 15;
struct Book {
    // you are accepting a race condition here because the user can update spread inbetween calculations
    ask_index: AtomicUsize,
    bid_index: AtomicUsize,
    table: [BookEntry; BOOK_SIZE],
}

// you will have to initially populate the book
static ORDER_BOOK: OnceLock<Book> = OnceLock::new();

pub fn limit_ask(
    price: u64,
    is_canceled: Arc<AtomicBool>,
    quantity: u32,
    money_address: Arc<AtomicU64>,
    sender: Sender<MarketOrder>,
) -> Result<(), SendError<MarketOrder>> {
    if (is_canceled.load(Ordering::Relaxed)) {
    } else {
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
    if (is_canceled.load(Ordering::Relaxed)) {
    } else {
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

static SYSTEM_END: AtomicBool = AtomicBool::new(false);

fn handle_ask(market_order: MarketOrder) {}

fn handle_bid(market_order: MarketOrder) {}

///pulls off queue and updates book should be its own thread
pub fn handle_orders(receiver: Receiver<MarketOrder>) {
    loop {
        //check for system end
        if (SYSTEM_END.load(Ordering::Relaxed) == true) {
            return;
        }

        // basically I am initially Receiving a result type then converting it in the next line to
        // either a market order or an error
        let market_order = receiver.recv();

        let market_order: MarketOrder = match market_order {
            Ok(market_order) => market_order,
            Err(error) => {
                println!("error receiving market order: {}", error);
                return;
            }
        };
        match market_order.order_type {
            OrderType::Ask => {
                handle_ask(market_order);
            }
            OrderType::Bid => {
                handle_bid(market_order);
            }
        }
    }
}
