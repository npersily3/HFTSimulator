use std::sync::atomic::{AtomicU32, AtomicU64,Ordering};
use std::sync::{Arc, OnceLock,LazyLock};
use crossbeam::queue::ArrayQueue;
use crossbeam::channel::{Sender,Receiver,bounded};





const ARRAY_SIZE: usize = 512;

struct PricePair {
    price: f32,
    quantity: u32,
}
struct Book {
     first_index: AtomicU32,
     table: [PricePair; ARRAY_SIZE],
}


struct GlobalBook {
    ask: Book,
    bid: Book,
}


//main market thread
    //generates price


enum OrderType {
    Ask,
    Bid,
}
struct MarketOrder {
    quantity: u32,
    // the user
    money_address:  Arc<AtomicU64>,
    order_type: OrderType,
}

static ORDER_QUEUE: LazyLock<ArrayQueue<MarketOrder>> = LazyLock::new( ||{
    ArrayQueue::new(ARRAY_SIZE)
});

//exchange thread functions

    //pub ask (quantity, address of money) void returns
pub fn ask( quantity:u32, money_address: Arc<AtomicU64> ) -> Result<(),MarketOrder> {
    let order = MarketOrder {
        quantity,
        money_address,
        order_type: OrderType::Ask
    };

    ORDER_QUEUE.push(order)
}
pub fn bid(quantity:u32, money_address: Arc<AtomicU64> ) -> Result<(),MarketOrder> {
    let order = MarketOrder {
        quantity,
        money_address,
        order_type: OrderType::Bid
    };

    ORDER_QUEUE.push(order)
}
    //pub bid (quantity, address of money)

        // it creates a market order
        // they both just push onto large circular arrays
        // the reason have the addresses is to update the users balance accordingly, when it eventually gets popped off the list
fn handle_orders() {
            loop {


            }


}

    //priv handle_orders
        // this is a thread excecuting in a loop that just pulls off of the circular arrays
        // then atomically updates the amount of money gained or lost at the memory address

