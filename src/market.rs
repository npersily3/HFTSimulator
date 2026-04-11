use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, LazyLock, RwLock};
use crossbeam::queue::ArrayQueue;
use crossbeam::channel::{Sender,Receiver,bounded, unbounded, SendError};





const ARRAY_SIZE: usize = 512;

struct PricePair {
    price: u64,
    quantity: u32,
}

impl Default for PricePair {
    fn default() -> Self {
        PricePair {price: 0, quantity: 0}
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




// Represents the two types of orders that can be on the exchange
enum MarketOrder {

    Ask { quantity: u32,
          price: u64,
          money_address: Arc<AtomicU64>
    },

    Bid {
        quantity: u32,
        money_address: Arc<AtomicU64> },
}

//exchange thread functions

    //pub ask (quantity, address of money) void returns
pub fn ask(quantity:u32, money_address: Arc<AtomicU64>,price: u64 ,sender: Sender<MarketOrder> ) -> Result<(), SendError<MarketOrder>> {
    let order = MarketOrder::Ask {
        quantity,
        price,
        money_address,
    };

    sender.send(order)
}
pub fn bid(quantity:u32, money_address: Arc<AtomicU64>,sender: Sender<MarketOrder> ) -> Result<(),SendError<MarketOrder>> {
    let order = MarketOrder::Bid {
        quantity,
        money_address,
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
            match market_order {
                MarketOrder::Ask { quantity, price, money_address} => {
                    let mut book = GLOBAL_BOOK.ask.write().unwrap();











                },

                MarketOrder::Bid { quantity, money_address} => {
                    let mut book = GLOBAL_BOOK.ask.write().unwrap();

                    let mut index = book.first_index;
                    let mut remaining = quantity;
                    let mut money_difference = 0;

                    // if we are consuming more than one row
                    while remaining > book.table[index].quantity {

                        // add the amount of money to the row as the quantity
                        money_difference += book.table[index].price * book.table[index].quantity as u64;

                        book.table[index].quantity = 0;


                        assert!(index > 0);
                        index -= 1;

                        assert!(quantity as i32 - (book.table[index].quantity as i32)< 0);
                        remaining -= book.table[index].quantity;

                    }

                    // if there is still a row left to complete
                    if quantity > 0 {

                        assert!(quantity < book.table[index].quantity);

                        let price = book.table[index].price;

                        //assume compiler magic will make this more than one assignment
                        money_difference += price * (quantity as u64);


                        //update the book to reflect the new quantity
                        book.table[index].quantity = book.table[index].quantity - quantity;
                        break;
                    }

                    book.first_index = index;
                    money_address.fetch_sub(money_difference, Ordering::Relaxed);
                }
            };



        }



    }
}


