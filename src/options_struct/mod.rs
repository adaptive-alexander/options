use crate::greeks::Greeks;
use crate::opt_data::OptData;
use crate::pricing_models::PricingModel;
use std::error::Error;
use std::path::PathBuf;

use crate::pricing_models::black_scholes::BlackScholesModel;
use csv::Writer;
use std::str::FromStr;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OptTypes {
    /// # OptTypes
    /// Enum to hold the two Option types call and put.
    Call,
    Put,
}

// Implementing trait FromStr to parse OptTypes
impl FromStr for OptTypes {
    /// # FromStr
    /// Implements FromStr to construct OptTypes from strings.
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase() as &str {
            // Case insensitive
            "call" => Ok(OptTypes::Call),
            "put" => Ok(OptTypes::Put),
            _ => Err(()),
        }
    }
}

// Implementing trait ToString to parse OptTypes
impl ToString for OptTypes {
    /// # ToString
    /// Implements ToString to output strings from OptTypes. Used for writing files.
    fn to_string(&self) -> String {
        match self {
            OptTypes::Put => "Put".to_string(),
            OptTypes::Call => "Call".to_string(),
        }
    }
}

pub struct Options {
    /// # Options
    /// A struct representing a financial options_old contract.
    pub opt_data: OptData,
    pub prices: Vec<f64>,
    pub greeks: Vec<Greeks>,
    model: Box<dyn PricingModel + Send>,
    iter_count: usize,
}

impl Options {
    /// # Options::new
    /// Literal construction method for Options
    /// # args:
    /// *`opt_data` - an [`OptData`] struct holding the necessary inputs to price an option.
    /// *`model` - Pricing model used to compute options_old. Has to implement PricingModel and Send.
    /// # returns:
    /// Returns an `Options` struct.
    pub fn new(opt_data: OptData, model: Box<dyn PricingModel + Send>) -> Self {
        Options {
            opt_data,
            prices: Vec::new(),
            greeks: Vec::new(),
            model,
            iter_count: 0,
        }
    }

    /// # Options::from_file
    /// Constructs options_old from file.
    ///
    /// # args:
    /// *`input_file` - Path to input file.
    /// *`model` - Pricing model used to compute options_old. Has to implement PricingModel and Send.

    /// # returns:
    /// Returns an `Options` struct.
    pub fn from_file(input_file: &PathBuf, model: Box<dyn PricingModel + Send>) -> Self {
        Options {
            opt_data: OptData::from_file(input_file),
            prices: Vec::new(),
            greeks: Vec::new(),
            model,
            iter_count: 0,
        }
    }

    /// # self.get_prices
    /// Computes prices based on model provided and stores in self.prices
    pub fn get_prices(&mut self) {
        self.prices = self.model.get_price(self);
    }

    /// # self.get_greeks
    /// Computes greeks based on model provided and stores in self.greeks
    pub fn get_greeks(&mut self) {
        self.greeks = self.model.get_greeks(self);
    }

    /// # self.to_records
    /// Flattens option data (deserialize to vector of flat records)
    ///
    /// # returns:
    /// A flattened representation of the data in a Vec<[String;16]>
    pub fn to_records(&self) -> Vec<[String; 16]> {
        if (self.prices.len() == 0) | (self.greeks.len() == 0) {
            panic!("Prices or Greeks of wrong length, or uninitialized.")
        }
        let mut records = Vec::with_capacity(self.opt_data.tickers.len());
        for i in 0..self.opt_data.tickers.len() {
            records.push([
                self.opt_data.tickers[i].clone(),
                self.opt_data.opt_types[i].to_string(),
                self.opt_data.underlying[i].to_string(),
                self.opt_data.strike[i].to_string(),
                self.opt_data.settles[i].to_string(),
                self.opt_data.maturities[i].to_string(),
                self.opt_data.duration[i].to_string(),
                self.opt_data.dividend[i].to_string(),
                self.opt_data.rfr[i].to_string(),
                self.opt_data.sigma[i].to_string(),
                self.prices[i].to_string(),
                self.greeks[i].delta.to_string(),
                self.greeks[i].gamma.to_string(),
                self.greeks[i].vega.to_string(),
                self.greeks[i].theta.to_string(),
                self.greeks[i].rho.to_string(),
            ])
        }
        records
    }

    /// # self.write_csv
    /// Writes flattened records out to csv
    pub fn write_csv(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
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

        // Collects chunked options_old back into one file, see function comment
        // Check Iterator implementation for what records contain.
        for rec in self.to_records() {
            wtr.write_record(rec)
                .expect("Failed writing file while iterating options_old.");
        }
        Ok(())
    }
}

impl Default for Options {
    /// # default
    /// Default method for initializing empty Options with Black-Scholes model
    ///
    /// # returns:
    /// An empty [`Options`] with a Black-Scholes model.
    fn default() -> Self {
        Options {
            opt_data: OptData::default(),
            prices: vec![],
            greeks: vec![],
            model: Box::new(BlackScholesModel::new()),
            iter_count: 0,
        }
    }
}
