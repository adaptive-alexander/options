use super::Options;
use super::PricingModel;
use crate::greeks::Greeks;
use crate::options_struct::OptTypes;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

/// # BlackScholesModel
/// Model to compute prices and greeks. Uses extended
/// Black-Scholes formula assuming continuous dividends.
/// For a better view of the mathematics review the notebook <https://github.com/adaptive-alexander/portfolio/blob/main/options/docs/notes.ipynb>.
pub struct BlackScholesModel;

/// # Implement Send for BlackScholesModel
/// Has to implement send to compute prices in parallel.
/// Required by Options trait object bounds.
unsafe impl Send for BlackScholesModel {}

impl BlackScholesModel {
    /// # BlackScholesModel::new
    /// Constructor method for BlackScholesModel
    ///
    /// # returns:
    /// Returns a BlackScholesModel
    pub fn new() -> Self {
        BlackScholesModel
    }

    /// # self.get_d1
    /// Computes the parameter d1
    ///
    /// # returns:
    /// An f64 value for d1
    fn get_d1(
        &self,
        underlying: &f64,
        strike: &f64,
        dividend: &f64,
        rfr: &f64,
        volatility: &f64,
        duration: &f64,
    ) -> f64 {
        (1.0 / (volatility * duration.sqrt()))
            * ((underlying / strike).ln()
                + duration * (rfr - dividend + (volatility.powf(2.0) / 2.0)))
    }

    /// # self.get_d1
    /// Computes parameter d2
    ///
    /// # returns:
    /// An f64 value for d2
    fn get_d2(&self, d1: &f64, volatility: &f64, duration: &f64) -> f64 {
        d1 - volatility * duration.sqrt()
    }
}

impl Default for BlackScholesModel {
    fn default() -> Self {
        BlackScholesModel
    }
}

impl PricingModel for BlackScholesModel {
    /// # self.get_price
    /// Computes prices
    ///
    /// # args:
    /// * `opts` - Takes a reference to options_old to use for calculations. This is passed self
    /// from [`Options`] get_price function.
    ///
    /// # returns:
    /// A vector of prices.
    fn get_price(&self, opt: &Options) -> Vec<f64> {
        // Initialize Standard Normal struct used to calculate distributions
        let n = Normal::new(0.0, 1.0).unwrap();

        // Initialize d1 and d2
        let mut d1: Vec<f64> = Vec::with_capacity(opt.opt_data.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(opt.opt_data.tickers.len());

        // Initialize return vector
        let mut prices = Vec::with_capacity(opt.opt_data.tickers.len());

        // Push d1 and d2
        for i in 0..opt.opt_data.tickers.len() {
            d1.push(self.get_d1(
                &opt.opt_data.underlying[i],
                &opt.opt_data.strike[i],
                &opt.opt_data.dividend[i],
                &opt.opt_data.rfr[i],
                &opt.opt_data.volatility[i],
                &opt.opt_data.duration[i],
            ));
            d2.push(self.get_d2(
                &d1[i],
                &opt.opt_data.volatility[i],
                &opt.opt_data.duration[i],
            ));

            // Push price into return Vec
            prices.push(
                // Compute price if call
                if opt.opt_data.opt_types[i] == OptTypes::Call {
                    opt.opt_data.underlying[i]
                        * (-opt.opt_data.dividend[i] * opt.opt_data.duration[i]).exp()
                        * n.cdf(d1[i])
                        - opt.opt_data.strike[i]
                            * (-opt.opt_data.rfr[i] * opt.opt_data.duration[i]).exp()
                            * n.cdf(d2[i])
                }
                // Compute price if put
                else if opt.opt_data.opt_types[i] == OptTypes::Put {
                    opt.opt_data.strike[i]
                        * (-opt.opt_data.rfr[i] * opt.opt_data.duration[i]).exp()
                        * n.cdf(-d2[i])
                        - opt.opt_data.underlying[i]
                            * (-opt.opt_data.dividend[i] * opt.opt_data.duration[i]).exp()
                            * n.cdf(-d1[i])
                } else {
                    // Enum only has above variants and is thus exhaustive.
                    panic!("This should not be able to happen, option type outside enum.")
                },
            )
        }
        prices
    }

    /// # self.get_greeks
    /// Computes option greeks
    ///
    /// # args:
    /// * `opts` - Takes a reference to options_old to use for calculations. This is passed self
    /// from [`Options`] get_price function.
    ///
    /// # returns:
    /// A vector of [`Greeks`].
    fn get_greeks(&self, opts: &Options) -> Vec<Greeks> {
        /// # get_delta
        /// Internal function used by get_greeks to compute option delta
        ///
        /// # args:
        /// *`n` - A [`Normal`] struct from statrs. Used to calculate normal distributions.
        /// *`opt_type` - Options type.
        /// *`d1` - d1 from [`get_d1`]
        /// *`dividend` - Option dividend, assumed to be continuous.
        /// *`duration` - Duration of options_old contract in years.
        ///
        /// # returns:
        /// Option delta (sensitivity to price changes)
        fn get_delta(
            n: &Normal,
            opt_type: &OptTypes,
            d1: &f64,
            dividend: &f64,
            duration: &f64,
        ) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (-(dividend * duration)).exp() * n.cdf(*d1)
            } else if *opt_type == OptTypes::Put {
                ret = (-(dividend * duration)).exp() * (n.cdf(*d1) - 1.0)
            }
            ret
        }

        /// # get_gamma
        /// Internal function used by get_greeks to compute option gamma
        ///
        /// # args:
        /// * `n` - A [`Normal`] struct from statrs. Used to calculate normal distributions.
        /// * `d1` - d1 from [`get_d1`]
        /// * `dividend` - Option dividend, assumed to be continuous.
        /// * `duration` - Duration of options_old contract in years.
        /// * `underlying` - Underlying price.
        /// * `volatility` - Annualized volatility.
        ///
        /// # returns:
        /// Option gamma (sensitivity of delta changes f'(delta))
        fn get_gamma(
            n: &Normal,
            d1: &f64,
            dividend: &f64,
            duration: &f64,
            underlying: &f64,
            volatility: &f64,
        ) -> f64 {
            (((-(dividend * duration)).exp()) / (underlying * volatility * duration.sqrt()))
                * n.pdf(*d1)
        }

        /// # get_vega
        /// Internal function used by get_greeks to compute option vega
        ///
        /// # args:
        /// * `n` - A [`Normal`] struct from statrs. Used to calculate normal distributions.
        /// * `d1` - d1 from [`get_d1`].
        /// * `dividend` - Option dividend, assumed to be continuous.
        /// * `duration` - Duration of options_old contract in years.
        /// * `underlying` - Underlying price.
        ///
        /// # returns:
        /// Option vega (sensitivity to volatility)
        fn get_vega(n: &Normal, d1: &f64, underlying: &f64, dividend: &f64, duration: &f64) -> f64 {
            (1.0 / 100.0)
                * underlying
                * (-(dividend * duration)).exp()
                * duration.sqrt()
                * n.pdf(*d1)
        }

        /// # get_theta
        /// Internal function used by get_greeks to compute option theta
        ///
        /// # args:
        /// * `n` - A [`Normal`] struct from statrs. Used to calculate normal distributions.
        /// * `d1` - d1 from [`get_d1`].
        /// * `d2` - d2 from [`get_d2`].
        /// * `opt_type` - Options type.
        /// * `dividend` - Option dividend, assumed to be continuous.
        /// * `duration` - Duration of options_old contract in years.
        /// * `strike` - Strike of options_old.
        /// * `rfr` - Risk free rate.
        /// * `underlying` - Underlying price.
        /// * `volatility` - Annualized volatility.
        ///
        /// # returns:
        /// Option theta (sensitivity to change in duration)
        fn get_theta(
            opt_type: &OptTypes,
            n: &Normal,
            d1: &f64,
            d2: &f64,
            underlying: &f64,
            dividend: &f64,
            duration: &f64,
            strike: &f64,
            rfr: &f64,
            volatility: &f64,
        ) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (1.0 / 365.25)
                    * (-(((underlying * volatility * (-(dividend * duration)).exp()) / 2.0
                        * duration.sqrt())
                        * n.pdf(*d1))
                        - rfr * strike * (-(rfr * duration)).exp() * n.cdf(*d2)
                        + dividend * underlying * (-(dividend * duration)).exp() * n.cdf(*d1))
            } else if *opt_type == OptTypes::Put {
                ret = (1.0 / 365.25)
                    * (-(((underlying * volatility * (-(dividend * duration)).exp()) / 2.0
                        * duration.sqrt())
                        * n.pdf(*d1))
                        + rfr * strike * (-(rfr * duration)).exp() * n.cdf(-*d2)
                        - dividend * underlying * (-(dividend * duration)).exp() * n.cdf(-*d1))
            }
            ret
        }

        /// # get_rho
        /// Internal function used by get_greeks to compute option rho
        ///
        /// # args:
        /// * `n` - A [`Normal`] struct from statrs. Used to calculate normal distributions.
        /// * `d2` - d2 from [`get_d2`].
        /// * `opt_type` - Options type.
        /// * `duration` - Duration of options_old contract in years.
        /// * `strike` - Strike of options_old.
        /// * `rfr` - Risk free rate.
        ///
        /// # returns:
        /// Option rho (sensitivity to interest rate changes)
        fn get_rho(
            n: &Normal,
            opt_type: &OptTypes,
            d2: &f64,
            strike: &f64,
            duration: &f64,
            rfr: &f64,
        ) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (1.0 / 100.0) * strike * duration * (-(rfr * duration)).exp() * n.cdf(*d2)
            } else if *opt_type == OptTypes::Put {
                ret = (-1.0 / 100.0) * strike * duration * (-(rfr * duration)).exp() * n.cdf(-*d2)
            }
            ret
        }

        // Driver code to construct Greeks
        // Initialize Normal struct used to compute distributions
        // Structure favors efficiency and sacrifices being verbose
        let n = Normal::new(0.0, 1.0).unwrap();

        // Initialize return Vec
        let mut gr = Vec::with_capacity(opts.opt_data.tickers.len());

        // Initialize Vecs for d1 and d2 values
        let mut d1: Vec<f64> = Vec::with_capacity(opts.opt_data.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(opts.opt_data.tickers.len());

        // Push values for d1 and d2
        for i in 0..opts.opt_data.tickers.len() {
            d1.push(self.get_d1(
                &opts.opt_data.underlying[i],
                &opts.opt_data.strike[i],
                &opts.opt_data.dividend[i],
                &opts.opt_data.rfr[i],
                &opts.opt_data.volatility[i],
                &opts.opt_data.duration[i],
            ));
            d2.push(self.get_d2(
                &d1[i],
                &opts.opt_data.volatility[i],
                &opts.opt_data.duration[i],
            ));
        }

        // Push greeks into return Vec
        for i in 0..opts.opt_data.tickers.len() {
            gr.push(Greeks {
                // get_delta
                delta: get_delta(
                    &n,
                    &opts.opt_data.opt_types[i],
                    &d1[i],
                    &opts.opt_data.dividend[i],
                    &opts.opt_data.duration[i],
                ),
                // get_gamma
                gamma: get_gamma(
                    &n,
                    &d1[i],
                    &opts.opt_data.dividend[i],
                    &opts.opt_data.duration[i],
                    &opts.opt_data.underlying[i],
                    &opts.opt_data.volatility[i],
                ),
                // get_vega
                vega: get_vega(
                    &n,
                    &d1[i],
                    &opts.opt_data.underlying[i],
                    &opts.opt_data.dividend[i],
                    &opts.opt_data.duration[i],
                ),
                // get_theta
                theta: get_theta(
                    &opts.opt_data.opt_types[i],
                    &n,
                    &d1[i],
                    &d2[i],
                    &opts.opt_data.underlying[i],
                    &opts.opt_data.dividend[i],
                    &opts.opt_data.duration[i],
                    &opts.opt_data.strike[i],
                    &opts.opt_data.rfr[i],
                    &opts.opt_data.volatility[i],
                ),
                // get_rho
                rho: get_rho(
                    &n,
                    &opts.opt_data.opt_types[i],
                    &d2[i],
                    &opts.opt_data.strike[i],
                    &opts.opt_data.duration[i],
                    &opts.opt_data.rfr[i],
                ),
            })
        }
        // Return Vec<Greeks>
        gr
    }
}
