use super::{Greeks, OptTypes, Options};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

pub struct BlackScholesModel;

impl BlackScholesModel {
    // Constructor method
    pub fn new() -> Self {
        BlackScholesModel
    }

    // Black-Scholes d1
    fn get_d1(
        &self,
        underlying: &f64,
        strike: &f64,
        dividend: &f64,
        rfr: &f64,
        sigma: &f64,
        duration: &f64,
    ) -> f64 {
        (1.0 / (sigma * duration.sqrt()))
            * ((underlying / strike).ln() + duration * (rfr - dividend + (sigma.powf(2.0) / 2.0)))
    }

    // Black-Scholes d2
    fn get_d2(&self, d1: &f64, sigma: &f64, duration: &f64) -> f64 {
        d1 - sigma * duration.sqrt()
    }

    // Black-Scholes option price
    pub fn bsm_price(&self, opt: &Options) -> Vec<f64> {
        // Initialize vectors
        let mut d1: Vec<f64> = Vec::with_capacity(opt.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(opt.tickers.len());
        let mut prices = Vec::with_capacity(opt.tickers.len());

        // Compute prices
        for i in 0..opt.tickers.len() {
            d1.push(self.get_d1(
                &opt.underlying[i],
                &opt.strike[i],
                &opt.dividend[i],
                &opt.rfr[i],
                &opt.sigma[i],
                &opt.duration[i],
            ));
            d2.push(self.get_d2(&d1[i], &opt.sigma[i], &opt.duration[i]));

            // Standard Normal struct
            let n = Normal::new(0.0, 1.0).unwrap();

            prices.push(if opt.opt_types[i] == OptTypes::Call {
                opt.underlying[i] * (-opt.dividend[i] * opt.duration[i]).exp() * n.cdf(d1[i])
                    - opt.strike[i] * (-opt.rfr[i] * opt.duration[i]).exp() * n.cdf(d2[i])
            } else if opt.opt_types[i] == OptTypes::Put {
                opt.strike[i] * (-opt.rfr[i] * opt.duration[i]).exp() * n.cdf(-d2[i])
                    - opt.underlying[i] * (-opt.dividend[i] * opt.duration[i]).exp() * n.cdf(-d1[i])
            } else {
                panic!("This should not be able to happen, option type outside enum.")
            })
        }
        prices
    }

    // todo!(Alexander): Comment this section
    // For a cleaner representation of the maths see repo notebook.
    pub fn bsm_greeks(&self, opts: &Options) -> Vec<Greeks> {
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

        fn get_gamma(
            n: &Normal,
            d1: &f64,
            dividend: &f64,
            duration: &f64,
            underlying: &f64,
            sigma: &f64,
        ) -> f64 {
            (((-(dividend * duration)).exp()) / (underlying * sigma * duration.sqrt())) * n.pdf(*d1)
        }

        fn get_vega(n: &Normal, d1: &f64, underlying: &f64, dividend: &f64, duration: &f64) -> f64 {
            (1.0 / 100.0)
                * underlying
                * (-(dividend * duration)).exp()
                * duration.sqrt()
                * n.pdf(*d1)
        }

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
            sigma: &f64,
        ) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (1.0 / 365.25)
                    * (-(((underlying * sigma * (-(dividend * duration)).exp()) / 2.0
                        * duration.sqrt())
                        * n.pdf(*d1))
                        - rfr * strike * (-(rfr * duration)).exp() * n.cdf(*d2)
                        + dividend * underlying * (-(dividend * duration)).exp() * n.cdf(*d1))
            } else if *opt_type == OptTypes::Put {
                ret = (1.0 / 365.25)
                    * (-(((underlying * sigma * (-(dividend * duration)).exp()) / 2.0
                        * duration.sqrt())
                        * n.pdf(*d1))
                        + rfr * strike * (-(rfr * duration)).exp() * n.cdf(-*d2)
                        - dividend * underlying * (-(dividend * duration)).exp() * n.cdf(-*d1))
            }
            ret
        }

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

        let n = Normal::new(0.0, 1.0).unwrap();
        let mut gr = Vec::with_capacity(opts.tickers.len());

        let mut d1: Vec<f64> = Vec::with_capacity(opts.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(opts.tickers.len());

        for i in 0..opts.tickers.len() {
            d1.push(self.get_d1(
                &opts.underlying[i],
                &opts.strike[i],
                &opts.dividend[i],
                &opts.rfr[i],
                &opts.sigma[i],
                &opts.duration[i],
            ));
            d2.push(self.get_d2(&d1[i], &opts.sigma[i], &opts.duration[i]));
        }

        for i in 0..opts.tickers.len() {
            gr.push(Greeks {
                delta: get_delta(
                    &n,
                    &opts.opt_types[i],
                    &d1[i],
                    &opts.dividend[i],
                    &opts.duration[i],
                ),
                gamma: get_gamma(
                    &n,
                    &d1[i],
                    &opts.dividend[i],
                    &opts.duration[i],
                    &opts.underlying[i],
                    &opts.sigma[i],
                ),
                vega: get_vega(
                    &n,
                    &d1[i],
                    &opts.underlying[i],
                    &opts.dividend[i],
                    &opts.duration[i],
                ),
                theta: get_theta(
                    &opts.opt_types[i],
                    &n,
                    &d1[i],
                    &d2[i],
                    &opts.underlying[i],
                    &opts.dividend[i],
                    &opts.duration[i],
                    &opts.strike[i],
                    &opts.rfr[i],
                    &opts.sigma[i],
                ),
                rho: get_rho(
                    &n,
                    &opts.opt_types[i],
                    &d2[i],
                    &opts.strike[i],
                    &opts.duration[i],
                    &opts.rfr[i],
                ),
            })
        }
        gr
    }
}
