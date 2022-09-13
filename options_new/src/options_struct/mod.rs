use crate::greeks::Greeks;
use crate::opt_data::OptData;
use crate::pricing_models::PricingModel;
use std::path::PathBuf;

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
    /// A struct representing a financial options contract.
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
    /// *`model` - Pricing model used to compute options. Has to implement PricingModel and Send.
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
    /// Constructs options from file.
    ///
    /// # args:
    /// *`input_file` - Path to input file.
    /// *`model` - Pricing model used to compute options. Has to implement PricingModel and Send.

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
}
