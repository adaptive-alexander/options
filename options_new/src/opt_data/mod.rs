use crate::options_struct::OptTypes;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

const SEC_YEAR: f64 = 60.0 * 60.0 * 24.0 * 365.25;

pub struct OptData {
    /// # OptData
    /// Struct to hold the input data needed to construct options.
    pub tickers: Vec<String>,
    pub opt_types: Vec<OptTypes>,
    pub underlying: Vec<f64>,
    pub strike: Vec<f64>,
    pub settles: Vec<DateTime<Utc>>,
    pub maturities: Vec<DateTime<Utc>>,
    pub duration: Vec<f64>,
    pub dividend: Vec<f64>,
    pub rfr: Vec<f64>,
    pub sigma: Vec<f64>,
}

impl OptData {
    /// # OptDat::new
    /// Literal constructor method for OptData
    ///
    /// # args:
    /// *`tickers` - Vector of strings containing tickers
    /// *`opt_types` - Vector of [`OptTypes`]
    /// *`underlying` - Vector of underlying prices.
    /// *`strike` - Vector of strike prices.
    /// *`settles` - Vector of settlement times using `chrono::Datetime`.
    /// *`maturities` - Vector of maturity times using `chrono::Datetime`.
    /// *`dividend` - Vector of dividends for the period. Make sure the
    /// dividends follow the same pattern the pricing model expects. For
    /// example Black-Scholes assumes continuous dividends for the period.
    /// *`rfr` - Vector fo risk free interest rate.
    /// *`sigma` - Vector of annualized volatility.
    ///
    /// # returns:
    /// Returns `OptData` struct.
    pub fn new(
        tickers: Vec<String>,
        opt_types: Vec<OptTypes>,
        underlying: Vec<f64>,
        strike: Vec<f64>,
        settles: Vec<DateTime<Utc>>,
        maturities: Vec<DateTime<Utc>>,
        dividend: Vec<f64>,
        rfr: Vec<f64>,
        sigma: Vec<f64>,
    ) -> Self {
        let mut opt_data = OptData {
            tickers,
            opt_types,
            underlying,
            strike,
            settles,
            maturities,
            duration: Vec::new(),
            dividend,
            rfr,
            sigma,
        };
        opt_data.duration = opt_data.get_durs();
        opt_data
    }

    /// # OptDat::from_file
    /// Literal constructor method for OptData
    ///
    /// # args:
    /// *`file` -  Path to input file.
    ///
    /// # returns:
    /// Returns `OptData` struct.
    pub fn from_file(file: &PathBuf) -> Self {
        unimplemented!()
    }

    /// # self.get_durs
    /// Get duration in years from settlement to maturity dates.
    ///
    /// # returns:
    /// Returns a vector of durations in years.
    fn get_durs(&self) -> Vec<f64> {
        let mut durs: Vec<f64> = Vec::with_capacity(self.settles.len());
        for i in 0..self.settles.len() {
            durs.push((self.maturities[i] - self.settles[i]).num_seconds() as f64 / SEC_YEAR)
        }
        durs
    }
}
