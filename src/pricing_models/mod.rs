pub mod black_scholes;

use crate::greeks::Greeks;
use crate::options_struct::Options;

/// # PricingModel
/// Trait required to pass a model to [`Options`].
pub trait PricingModel {
    fn get_price(&self, opts: &Options) -> Vec<f64>;
    fn get_greeks(&self, opts: &Options) -> Vec<Greeks>;
}
