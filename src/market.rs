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
     first_index: usize,
     table: [PricePair; ARRAY_SIZE],
}
impl Default for Book {
    fn default() -> Self {
        Book {first_index: (ARRAY_SIZE/2), table: std::array::from_fn(|_| PricePair::default())}
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




static GLOBAL_BOOK: LazyLock<GlobalBook> = LazyLock::new(|| {
    GlobalBook {
        ask: RwLock::new(Book::default()),
        bid: RwLock::new(Book::default()),
    }
});

///pulls off queue and updates book should be its own thread
pub fn handle_orders(receiver: Receiver<MarketOrder>) {
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

        // it being in its own scope allows the lock to be released
        // This block updates it from the user side
        {
            // make sure it is locked
            let mut book  = match market_order.order_type {
                OrderType::Ask => &mut GLOBAL_BOOK.ask.write().unwrap(),
                OrderType::Bid => &mut GLOBAL_BOOK.bid.write().unwrap(),
            };

            let mut index = book.first_index;
            let mut quantity = market_order.quantity;
            let mut money_difference:f32 = 0.0;

            // if we are consuming more than one row
            while quantity > book.table[index].quantity {

                // add the amount of money to the row as the quantity
                money_difference += book.table[index].price * (book.table[index].quantity as f32);

                book.table[index].quantity = 0;


                assert!(index > 0);
                index -= 1;

                assert!(quantity as i32 - (book.table[index].quantity as i32)< 0);
                quantity -= book.table[index].quantity;

            }

            // if there is still a row left to complete
            if quantity > 0 {

                assert!(quantity < book.table[index].quantity);

                let price = book.table[index].price;

                //assume compiler magic will make this more than one assignment
                money_difference += price * (quantity as f32);

                let money_difference = match market_order.order_type {
                    OrderType::Bid => -1.0 * money_difference,
                    OrderType::Ask => money_difference,
                };
                //update the book to reflect the new quantity
                book.table[index].quantity = book.table[index].quantity - quantity;
                break;
            }

            book.first_index = index;

        }

        // if the stock is sold back we shoyuld probably do something
        if(matches!(market_order.order_type, OrderType::Ask)) {

        }

    }
}


