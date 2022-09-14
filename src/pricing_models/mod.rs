pub mod black_scholes;

use crate::greeks::Greeks;
use crate::options_struct::Options;

pub trait PricingModel {
    fn get_price(&self, opts: &Options) -> Vec<f64>;
    fn get_greeks(&self, opts: &Options) -> Vec<Greeks>;
}
