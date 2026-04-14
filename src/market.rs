use std::ops::Deref;
use crate::exchange::{Book, MarketOrder};
use crate::{exchange, utils};
use crossbeam::channel::Sender;
use rand::rngs::ThreadRng;
use rand_distr::{Exp, Exp1, LogNormal};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::sleep;
use std::time::Duration;
use rand::distr::Distribution;
use rand::prelude::*;

pub const INITIAL_MONEY: u64 = 1000000;

pub struct  QuantitySampler {
    log_normal: LogNormal<f64>,
    rng: ThreadRng,
}

impl QuantitySampler {
    fn new(mean: f64, std: f64) -> QuantitySampler {
        let variance = std * std;
        let sigma2 = (1.0 + variance / (mean * mean)).ln();
        let sigma = sigma2.sqrt();
        let mu = mean.ln() - sigma2 / 2.0;

        Self {
            log_normal: LogNormal::new(mu, sigma).unwrap(),
            rng: ThreadRng::default(),
        }
    }

    fn sample (&mut self) -> u32 {
        let result = self.log_normal.sample(&mut self.rng) as u64;
        std::cmp::min(result, 100) as u32
    }
}

pub struct SleepSampler {
    rng: ThreadRng,


}
impl SleepSampler {
    fn new() -> Self {



        let rng = ThreadRng::default();

        Self {
            rng,

        }

    }
    fn sample (&mut self) -> f64 {
       let x: f64 = self.rng.sample(Exp1);
          x * SLEEP_MEAN_IN_MS
    }
}

const QUANTITY_MEAN: f64 = 10.0;
// use
const QUANTITY_STD: f64 = 20.0;
const SLEEP_MEAN_IN_MS: f64 = 100.0;
const ORDER_PROBABILITY: f64 = 0.1;

pub fn noise(sender: Sender<MarketOrder>, start: Arc<Barrier>, tick: Option<Arc<Barrier>>)  {

    let money = Arc::new(AtomicU64::new(INITIAL_MONEY));
    let mut quantity_sampler = QuantitySampler::new(QUANTITY_MEAN, QUANTITY_STD);
    let mut sleep_sampler = SleepSampler::new();
    let is_canceled = Arc::new(AtomicBool::new(false));

    start.wait();

    loop {
        if utils::SYSTEM_END.load(Ordering::Relaxed) {
            break;
        }

        match tick.as_ref() {
            Some(tick) => {
               if rand::random::<f64>() < ORDER_PROBABILITY {
                    continue;
                }
            }
            None => {}
        }

        let quantity = quantity_sampler.sample();

        let is_ask = rand::random::<bool>();

        if is_ask {
            match exchange::ask(is_canceled.clone(), quantity, money.clone(), sender.clone()) {
                Ok(_) => {}
                Err(e) => {println!("order rejected")}
            }
        } else {
            match exchange::bid(is_canceled.clone(), quantity, money.clone(), sender.clone()) {
                Ok(_) => {}
                Err(e) => {println!("order rejected")}
            }
        }
        match tick.as_ref() {
            Some(tick) => {
                tick.wait();
            }
            None => {
                let duration = sleep_sampler.sample() as u64;
                sleep(Duration::from_millis(duration));
            }
        }
    }



    let final_money = money.load(Ordering::Relaxed);

}

const THRESHOLD: i64 = 50;
pub fn fundamentalist(sender: Sender<MarketOrder>, start: Arc<Barrier>, tick: Option<Arc<Barrier>>, ask_index: Arc<AtomicUsize>, bid_index: Arc<AtomicUsize>, true_price: Arc<AtomicU64>) {
    let money = Arc::new(AtomicU64::new(INITIAL_MONEY));

    let is_canceled = Arc::new(AtomicBool::new(false));
    start.wait();

    loop {

        if utils::SYSTEM_END.load(Ordering::Relaxed) {
            break;
        }


        let mid_price = ((ask_index.load(Ordering::Relaxed) + bid_index.load(Ordering::Relaxed))/2) as i64;
        let true_price = true_price.load(Ordering::Relaxed) as i64;

        if (mid_price - true_price).abs() > THRESHOLD {
            // this means that the stock is overvalued so they should sell
            match mid_price > true_price {
                true => {
                    exchange::limit_ask(true_price as u64 ,is_canceled.clone(),10, money.clone(), sender.clone()).unwrap();
                }
                false => {
                    exchange::limit_bid(true_price as u64 ,is_canceled.clone(),10, money.clone(), sender.clone()).unwrap();
                }
            }

        }

        match tick.as_ref() {
            Some(tick) => {
                println!("entered tick");
                tick.wait();
            }
            None => {

                sleep(Duration::from_millis(200));
            }
        }
    }

    println!("money: {} in cents", money.load(Ordering::Relaxed));
}