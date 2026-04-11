use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, LazyLock, RwLock};
use crossbeam::queue::ArrayQueue;
use crossbeam::channel::{Sender,Receiver,bounded, unbounded, SendError};





const ARRAY_SIZE: usize = 512;

struct PricePair {
    price: f32,
    quantity: u32,
}

impl Default for PricePair {
    fn default() -> Self {
        PricePair {price: 0.0, quantity: 0}
    }
}

struct Book {
     first_index: u32,
     table: [PricePair; ARRAY_SIZE],
}
impl Default for Book {
    fn default() -> Self {
        Book {first_index: (ARRAY_SIZE/2) as u32, table: std::array::from_fn(|_| PricePair::default())}
    }
}

struct GlobalBook {
    ask: RwLock<Book>,
    bid: RwLock<Book>,
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



//exchange thread functions

    //pub ask (quantity, address of money) void returns
pub fn ask(quantity:u32, money_address: Arc<AtomicU64>, sender: Sender<MarketOrder> ) -> Result<(), SendError<MarketOrder>> {
    let order = MarketOrder {
        quantity,
        money_address,
        order_type: OrderType::Ask
    };

    sender.send(order)
}
pub fn bid(quantity:u32, money_address: Arc<AtomicU64>,sender: Sender<MarketOrder> ) -> Result<(),SendError<MarketOrder>> {
    let order = MarketOrder {
        quantity,
        money_address,
        order_type: OrderType::Bid
    };

   sender.send(order)
}
    //pub bid (quantity, address of money)

        // it creates a market order
        // they both just push onto large circular arrays
        // the reason have the addresses is to update the users balance accordingly, when it eventually gets popped off the list

        static SYSTEM_END: AtomicBool = AtomicBool::new(false);

//priv handle_orders
// this is a thread excecuting in a loop that just pulls off of the circular arrays
// then atomically updates the amount of money gained or lost at the memory address
//TODO we have to determine how the data is shared better


static GLOBAL_BOOK: LazyLock<GlobalBook> = LazyLock::new(|| {
    GlobalBook {
        ask: RwLock::new(Book::default()),
        bid: RwLock::new(Book::default()),
    }
});
fn handle_orders(receiver: Receiver<MarketOrder>) {
    loop {
        //check for system end
        if(SYSTEM_END.load(Ordering::Relaxed) == true) {
            return;
        }

        // basically I am initially Receiving a result type then converting it in the next line to
        // either a market order or an error
        let market_order =  receiver.recv();

        let market_order = match market_order {
            Ok(market_order) => market_order,
            Err(error) => {
                println!("error receiving market order: {}", error);
                return;
            },
        };

        let book : &RwLock<Book> = match market_order.order_type {
            OrderType::Ask => & GLOBAL_BOOK.ask,
            OrderType::Bid => & GLOBAL_BOOK.bid,
        };





    }


}


