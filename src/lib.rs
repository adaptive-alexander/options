#![allow(non_snake_case)]

use chrono::offset::Utc;
use chrono::DateTime;
use csv::Writer;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;

mod black_scholes;
pub mod parse_input;

const SEC_YEAR: f64 = 60.0 * 60.0 * 24.0 * 365.25;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OptTypes {
    Call,
    Put,
}

// Implementing trait FromStr to parse OptTypes
impl FromStr for OptTypes {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase() as &str {
            "call" => Ok(OptTypes::Call),
            "put" => Ok(OptTypes::Put),
            _ => Err(()),
        }
    }
}

// Implementing trait ToString to parse OptTypes
impl ToString for OptTypes {
    fn to_string(&self) -> String {
        match self {
            OptTypes::Put => "Put".to_string(),
            OptTypes::Call => "Call".to_string(),
        }
    }
}

// Struct for option greeks
// Separated both for structure and to keep ability to initialize separately
// if computing prices without greeks for a large number of options
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

// Default implementation for option greeks (need to be initialized in struct)
impl Default for Greeks {
    /// Returns default Black-Scholes Greeks initialized as 0
    /// Only intended to be used by init function as placeholder
    fn default() -> Self {
        Greeks {
            delta: 0.0,
            gamma: 0.0,
            vega: 0.0,
            theta: 0.0,
            rho: 0.0,
        }
    }
}

impl fmt::Display for Greeks {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        str = format!("{} Delta: {:.4} \n", str, self.delta)
            .parse() // To output Result type
            .unwrap();
        str = format!("{} Gamma: {:.4} \n", str, self.gamma)
            .parse()
            .unwrap();
        str = format!("{} Vega: {:.4} \n", str, self.vega)
            .parse()
            .unwrap();
        str = format!("{} Theta: {:.4} \n", str, self.theta)
            .parse()
            .unwrap();
        str = format!("{} Rho: {:.4}", str, self.rho).parse().unwrap();
        write!(f, "{}", str)
    }
}

pub struct Options {
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
    pub prices: Vec<f64>,
    pub greeks: Vec<Greeks>,
    iter_count: usize,
}

// Needed to write out
impl Iterator for Options {
    type Item = [String; 16]; // Note that the iterator outputs String

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.iter_count;
        self.iter_count += 1;
        if self.iter_count <= self.tickers.len() {
            Some([
                self.tickers[i].clone(),
                self.opt_types[i].to_string(),
                self.underlying[i].to_string(),
                self.strike[i].to_string(),
                self.settles[i].to_string(),
                self.maturities[i].to_string(),
                self.duration[i].to_string(),
                self.dividend[i].to_string(),
                self.rfr[i].to_string(),
                self.sigma[i].to_string(),
                self.prices[i].to_string(),
                self.greeks[i].delta.to_string(),
                self.greeks[i].gamma.to_string(),
                self.greeks[i].vega.to_string(),
                self.greeks[i].theta.to_string(),
                self.greeks[i].rho.to_string(),
            ])
        } else {
            None
        }
    }
}

// todo!(Alexander): add argument to pass pricing model with Black-Scholes as default.
impl Options {
    // Constructor pattern
    /// # Constructor method for Options
    /// Initializes Options with Black-Scholes prices and Default Greeks with 0.0 values.
    ///
    /// # Arguments
    ///
    /// * `tickers` - Vector of Strings holding ticker names.
    /// * `opt_types` - Vector of enum OptTypes holding option type.
    /// * `underlying` - Vector of f64 underlying prices.
    /// * `strike` - Vector of f64 strike prices.
    /// * `settles` - Vector of `chrono` settlement times.
    /// * `maturities` - Vector of `chrono` maturity times.
    /// * `dividend` - Vector of f64 dividends.
    /// * `rfr` - Vector of f64 risk free rates.
    /// * `sigma` - Vector of f64 volatilities.
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
        let len = &tickers.len();
        let mut opts = Options {
            tickers,
            opt_types,
            underlying,
            strike,
            settles,
            maturities,
            duration: Vec::with_capacity(*len),
            dividend,
            rfr,
            sigma,
            prices: Vec::with_capacity(*len),
            greeks: Vec::with_capacity(*len),
            iter_count: 0,
        };
        opts.init_vals();
        opts
    }

    // Initialize option values
    // Note that it also initializes greeks as default
    fn init_vals(&mut self) {
        self.duration
            .append(&mut get_durs(&self.settles, &self.maturities));
        // Initialize Black-Scholes model
        let bsm_model = black_scholes::BlackScholesModel::new();
        self.prices = bsm_model.bsm_price(self);
        for _ in 0..self.tickers.len() {
            self.greeks.push(Greeks {
                ..Greeks::default()
            })
        }
    }
}

fn get_durs(settles: &[DateTime<Utc>], maturities: &[DateTime<Utc>]) -> Vec<f64> {
    let mut durs: Vec<f64> = Vec::with_capacity(settles.len());
    for i in 0..settles.len() {
        durs.push((maturities[i] - settles[i]).num_seconds() as f64 / SEC_YEAR)
    }
    durs
}

pub fn initialize_opts(
    tup: (
        Vec<String>,
        Vec<OptTypes>,
        Vec<f64>,
        Vec<f64>,
        Vec<DateTime<Utc>>,
        Vec<DateTime<Utc>>,
        Vec<f64>,
        Vec<f64>,
        Vec<f64>,
    ),
) -> Options {
    let bsm_model = black_scholes::BlackScholesModel;

    // Initialize Options
    let mut opts = Options::new(
        tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6, tup.7, tup.8,
    );

    // Initialize greeks
    opts.greeks = bsm_model.bsm_greeks(&opts);

    // Return Options with greeks
    opts
}

// Writing Options struct to csv
// Note arg opts: Vec<Options>. Reason: Rayon has chunked data for computation into Vec<Options> in options-listener
// Combining earlier inefficient. Alternative is to append chunked Vecs after computation
// todo!(Alexander): Check if file exists. Currently overwrites content (not file). N.B. input files from Python timestamped, minor impact.
pub fn write_csv_out(path: PathBuf, opts: Vec<Options>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    // Column headers
    let headers = [
        "ticker",
        "opt_type",
        "underlying",
        "strike",
        "settle",
        "maturity",
        "duration",
        "dividend",
        "rfr",
        "sigma",
        "price",
        "delta",
        "gamma",
        "vega",
        "theta",
        "rho",
    ];
    wtr.write_record(headers).expect("failed writing headers");

    // Collects chunked options back into one file, see function comment
    for opt in opts {
        for rec in opt {
            wtr.write_record(rec)
                .expect("Failed writing file while iterating options.");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeks_default() {
        let greeks = Greeks::default();
        assert_eq!(
            0.0,
            greeks.delta + greeks.gamma + greeks.rho + greeks.theta + greeks.vega
        );
    }
}
