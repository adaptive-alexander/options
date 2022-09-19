use crate::options_struct::OptTypes;
use crate::utilities::retry_open_file;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const SEC_YEAR: f64 = 60.0 * 60.0 * 24.0 * 365.25;

pub struct OptData {
    /// # OptData
    /// Struct to hold the input data needed to construct options_old.
    pub tickers: Vec<String>,
    pub opt_types: Vec<OptTypes>,
    pub underlying: Vec<f64>,
    pub strike: Vec<f64>,
    pub settles: Vec<DateTime<Utc>>,
    pub maturities: Vec<DateTime<Utc>>,
    pub duration: Vec<f64>,
    pub dividend: Vec<f64>,
    pub rfr: Vec<f64>,
    pub volatility: Vec<f64>,
}

impl OptData {
    /// # OptDat::new
    /// Literal constructor method for OptData
    ///
    /// # args:
    /// * `tickers` - Vector of strings containing tickers
    /// * `opt_types` - Vector of [`OptTypes`]
    /// * `underlying` - Vector of underlying prices.
    /// * `strike` - Vector of strike prices.
    /// * `settles` - Vector of settlement times using `chrono::Datetime`.
    /// * `maturities` - Vector of maturity times using `chrono::Datetime`.
    /// * `dividend` - Vector of dividends for the period. Make sure the
    /// dividends follow the same pattern the pricing model expects. For
    /// example Black-Scholes assumes continuous dividends for the period.
    /// * `rfr` - Vector fo risk free interest rate.
    /// * `volatility` - Vector of annualized volatility.
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
        volatility: Vec<f64>,
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
            volatility,
        };
        opt_data.duration = opt_data.get_durs();
        opt_data
    }

    /// # OptDat::from_file
    /// Literal constructor method for OptData
    ///
    /// # args:
    /// * `file` -  Path to input file.
    ///
    /// # returns:
    /// Returns `OptData` struct.
    pub fn from_file(file: &PathBuf) -> Self {
        let tup = parse_input(file);
        OptData::new(
            tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6, tup.7, tup.8,
        )
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

impl Default for OptData {
    /// # default
    /// Default method for initializing empty OptData
    ///
    /// # returns:
    /// An empty [`OptData`]
    fn default() -> Self {
        OptData {
            tickers: vec![],
            opt_types: vec![],
            underlying: vec![],
            strike: vec![],
            settles: vec![],
            maturities: vec![],
            duration: vec![],
            dividend: vec![],
            rfr: vec![],
            volatility: vec![],
        }
    }
}

/// # parse_date
/// Parses string dates
///
/// # args:
/// * `s` - A string to parse
///
/// # returns:
/// A chrono compliant string as long as parsing was successful.
// todo!("stability: add error type if parse unsuccessful")
fn parse_date(s: &str) -> String {
    let mut s_ret;

    // The following patterns handle most of Pythons native date types
    // Regex used to search
    match &s.find('+') {
        Some(_) => return s.to_string(),
        None => {}
    };
    match &s.find(r"-\d{2}:\d{2}") {
        Some(_) => return s.to_string(),
        None => {}
    };
    match &s.find('t') {
        Some(_) => s_ret = format!("{}{}", s, "+00:00"),
        None => s_ret = format!("{}{}{}", s, "t00:00:00", "+00:00"),
    };
    match &s.find(' ') {
        Some(i) => {
            s_ret = format!(
                "{}{}{}{}",
                s.to_string().get(0..*i).unwrap(),
                "t",
                s.to_string().get(*i + 1..*i + 9).unwrap(),
                "+00:00"
            )
        }
        None => {}
    }
    s_ret
}

/// # read_lines
/// Convenience function to get lines of file
///
/// # args:
/// * `filename` - Path to file.
///
/// # returns:
/// Returnes a buffer containing the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// # parse_input
/// Parses a file for OptData inputs
///
/// # args:
/// * `path` - Path to the file to parse.
///
/// # returns:
/// A tuple of vectors used to initialize [`OptData`]
pub fn parse_input(
    path: &PathBuf,
) -> (
    Vec<String>,
    Vec<OptTypes>,
    Vec<f64>,
    Vec<f64>,
    Vec<DateTime<Utc>>,
    Vec<DateTime<Utc>>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
) {
    // Initializing variables
    let mut file;
    let mut lines_num;

    // Compute number of lines
    loop {
        file = retry_open_file(path);
        // parse number of lines in file
        lines_num = 0;
        for _ in file.lines().skip(1) {
            lines_num += 1;
        }
        if lines_num > 0 {
            break;
        }
    }

    println!("Processing {} options", &lines_num);

    // Get index position of column containing appropriate data
    if let Ok(mut lines) = read_lines(&path) {
        let procc_s = lines.next().unwrap().unwrap();
        let headers: Vec<&str> = procc_s.split(',').collect();
        let tick_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "ticker")
            .expect("No header tickers in file");
        let opt_t_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "opt_type")
            .expect("No header opt_type in file");
        let underlying_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "underlying")
            .expect("No header underlying in file");
        let strike_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "strike")
            .expect("No header strike in file");
        let set_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "settle")
            .expect("No header settle in file");
        let mat_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "maturity")
            .expect("No header maturity in file");
        let dividend_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "dividend")
            .expect("No header dividend in file");
        let rfr_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "rfr")
            .expect("No header rfr in file");
        let volatility_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "volatility")
            .expect("No header volatility in file");

        // initializing Vectors
        let mut tickers: Vec<String> = Vec::with_capacity(lines_num);
        let mut opt_types: Vec<OptTypes> = Vec::with_capacity(lines_num);
        let mut underlying: Vec<f64> = Vec::with_capacity(lines_num);
        let mut strike: Vec<f64> = Vec::with_capacity(lines_num);
        let mut settles: Vec<DateTime<Utc>> = Vec::with_capacity(lines_num);
        let mut maturities: Vec<DateTime<Utc>> = Vec::with_capacity(lines_num);
        let mut dividend: Vec<f64> = Vec::with_capacity(lines_num);
        let mut rfr: Vec<f64> = Vec::with_capacity(lines_num);
        let mut volatility: Vec<f64> = Vec::with_capacity(lines_num);

        // push data
        for line in lines.flatten() {
            let inps: Vec<&str> = line.split(',').collect();
            tickers.push(inps[tick_idx].to_string());
            opt_types.push(OptTypes::from_str(inps[opt_t_idx]).unwrap());
            underlying.push(
                inps[underlying_idx]
                    .parse::<f64>()
                    .expect("failed to parse s to f64"),
            );
            strike.push(
                inps[strike_idx]
                    .parse::<f64>()
                    .expect("failed to parse k to f64"),
            );
            settles.push(DateTime::from(
                DateTime::parse_from_rfc3339(&*parse_date(inps[set_idx])).unwrap(),
            ));
            maturities.push(DateTime::from(
                DateTime::parse_from_rfc3339(&*parse_date(inps[mat_idx])).unwrap(),
            ));
            dividend.push(
                inps[dividend_idx]
                    .parse::<f64>()
                    .expect("failed to parse s to f64"),
            );
            rfr.push(
                inps[rfr_idx]
                    .parse::<f64>()
                    .expect("failed to parse s to f64"),
            );
            volatility.push(
                inps[volatility_idx]
                    .parse::<f64>()
                    .expect("failed to parse s to f64"),
            );
        }
        // Return tuple of columns
        (
            tickers, opt_types, underlying, strike, settles, maturities, dividend, rfr, volatility,
        )
    } else {
        panic!("Unable to parse input.")
    }
}
