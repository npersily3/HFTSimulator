use crossbeam::channel::{Receiver, SendError, Sender, bounded, unbounded};
use crossbeam::queue::ArrayQueue;
use std::cell::LazyCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use std::time::Instant;

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
    order_type: OrderType,
    price: u64,
    quantity: u32,
    timestamp: Instant,
}
const HISTORY_SIZE: usize = 1 << 15;
static MARKET_HISTORY: LazyLock<ArrayQueue<HistoryEntry>> =
    LazyLock::new(|| ArrayQueue::new(HISTORY_SIZE));

struct BookEntry {
    bids: VecDeque<MarketOrder>,
    asks: VecDeque<MarketOrder>,
}

impl BookEntry {
    fn new() -> Self {
        let bids: VecDeque<MarketOrder> = VecDeque::new();
        let asks: VecDeque<MarketOrder> = VecDeque::new();

        let book_entry = BookEntry { bids, asks };
        book_entry
    }
}

const BOOK_SIZE: usize = 1 << 15;
pub struct Book {
    // you are accepting a race condition here because the user can update spread inbetween calculations
    ask_index: AtomicUsize,
    bid_index: AtomicUsize,
    table: [BookEntry; BOOK_SIZE],
}
impl Book {
    //TODO
    fn new() -> Book {
        let mut table: [BookEntry; BOOK_SIZE] = std::array::from_fn(|_| BookEntry::new());
        let ask_index = AtomicUsize::new(0);
        let bid_index = AtomicUsize::new(10);

        let book = Book {
            ask_index,
            bid_index,
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

fn handle_ask(market_order: MarketOrder, sender: &Sender<HistoryEntry>, order_book: &mut Book) {
    let mut index = order_book.bid_index.load(Ordering::Relaxed);

    //create local mutables of the order
    let mut price = market_order.price;
    let mut ask_quantity = market_order.quantity;

    //TODO handle non limit orders better
    //traversing from the highest to lowest bid we are willing to go to
    while index >= price as usize {
        //if we have any bids at the current price
        while order_book.table[index].bids.is_empty() == false {
            let current_bid = order_book.table[index].bids.front_mut();
            debug_assert!(current_bid.is_some());
            let current_bid = current_bid.unwrap();

            // if it was cancelled pop it off and move over
            if (current_bid.is_canceled.load(Ordering::Relaxed)) {
                order_book.table[index].bids.pop_front();
                continue;
            }
            // let mut money_difference = 0;
            // if(current_bid.quantity > ask_quantity) {
            //     current_bid.quantity -= ask_quantity;
            //     money_difference = ask_quantity as u64 * index as u64;
            // } else {
            //     ask_quantity -= current_bid.quantity;
            //     money_difference = money_difference * index as u64;
            // }



            // if I am not going to consume another entry
            if (current_bid.quantity >= ask_quantity) {
                current_bid.quantity -= ask_quantity;

                let money_difference = ask_quantity as u64 * (index as u64);

                market_order.money_address.fetch_add(money_difference, Ordering::Relaxed);
                current_bid.money_address.fetch_sub(money_difference, Ordering::Relaxed);

                let record = HistoryEntry {
                    order_type: OrderType::Ask,
                    price: money_difference,
                    quantity: market_order.quantity,
                    timestamp: Instant::now(),
                };
                let sender_status = sender.send(record);
                match sender_status {
                    Ok(_) => {}
                    Err(_) => {
                        panic!("history channel disconnected")
                    }
                }

                // if took exactly the last amount pop it off
                if (current_bid.quantity == 0) {
                    order_book.table[index].bids.pop_front();
                }

                return;
            } else {
                ask_quantity -= current_bid.quantity;

                let money_difference = current_bid.quantity as u64 * (index as u64);

                market_order.money_address.fetch_add(money_difference, Ordering::Relaxed);
                current_bid.money_address.fetch_sub(money_difference, Ordering::Relaxed);

                let record = HistoryEntry {
                    order_type: OrderType::Ask,
                    price: money_difference,
                    quantity: market_order.quantity,
                    timestamp: Instant::now(),
                };
                let sender_status = sender.send(record);
                match sender_status {
                    Ok(_) => {}
                    Err(_) => {
                        panic!("history channel disconnected")
                    }
                }

                //remove the entry we just ran through
                order_book.table[index].bids.pop_front();
            }
        }

        index -= 1;
    }
}

fn handle_bid(market_order: MarketOrder, sender: &Sender<HistoryEntry>, order_book: &mut Book) {}

///pulls off queue and updates book should be its own thread
pub fn handle_orders(
    receiver: Receiver<MarketOrder>,
    sender: Sender<HistoryEntry>,
    mut order_book: Book,
) {
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
                handle_ask(market_order, &sender, &mut order_book);
            }
            OrderType::Bid => {
                handle_bid(market_order, &sender, &mut order_book);
            }
        }
    }
}
