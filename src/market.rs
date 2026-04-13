use crate::exchange::MarketOrder;
use crate::{exchange, utils};
use crossbeam::channel::Sender;
use rand::rngs::ThreadRng;
use rand_distr::{Exp, Exp1, LogNormal};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use rand::distr::Distribution;
use rand::prelude::*;

const INITIAL_MONEY: u64 = 1000000;

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

fn get_noise_quantity(quantity_sampler: &mut QuantitySampler) -> u32 {
    quantity_sampler.sample()
}

pub struct SleepSampler {
    rng: ThreadRng,

}
impl SleepSampler {
    fn new() -> Self {



        let rng = ThreadRng::default();

        Self {

            rng
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


fn noise(sender: Sender<MarketOrder>, start: Arc<Barrier>, tick: Option<Arc<Barrier>>)  {

    let money = Arc::new(AtomicU64::new(INITIAL_MONEY));
    let mut quantity_sampler = QuantitySampler::new(QUANTITY_MEAN, QUANTITY_STD);
    let is_canceled = Arc::new(AtomicBool::new(false));

    start.wait();

    loop {
        if(utils::SYSTEM_END.load(Ordering::Relaxed)) {
            break;
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

    }



    let final_money = money.load(Ordering::Relaxed);

}