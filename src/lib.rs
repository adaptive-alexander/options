#![allow(non_snake_case)]

use chrono::offset::Utc;
use chrono::DateTime;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use csv::Writer;
use std::error::Error;
use std::io;

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
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

// Default implementation for option greeks (need to be initialized in struct)
impl Default for Greeks {
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
            .parse()                        // To output Result type
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
    type Item = [String; 16];  // Note that the iterator outputs String

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

// todo!(Alexander): remove clones, pass by value
impl Options {
    // Constructor pattern
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
        let mut opts = Options {
            tickers: tickers.clone(),  // Has to own values
            opt_types,
            underlying,
            strike,
            settles: settles.clone(),  // Has to own values
            maturities: maturities.clone(),  // Has to own values
            duration: get_durs(&settles, &maturities),
            dividend,
            rfr,
            sigma,
            prices: Vec::with_capacity(tickers.len()),
            greeks: Vec::with_capacity(tickers.len()),
            iter_count: 0,
        };
        opts.init_vals();
        opts
    }

    // Initialize option values
    // Note that it also initializes greeks as default
    fn init_vals(&mut self) {
        self.prices = self.bsm_price();
        for _ in 0..self.tickers.len() {
            self.greeks.push(Greeks {
                ..Greeks::default()
            })
        }
    }

    // Black-Scholes d1 and d2 here instead of inside bsm_price because greeks use them
    // Black-Scholes d1
    fn get_d1(&self, underlying: &f64, strike: &f64, dividend: &f64, rfr: &f64, sigma: &f64, duration: &f64) -> f64 {
        (1.0 / (sigma * duration.sqrt())) * ((underlying / strike).ln() + duration * (rfr - dividend + (sigma.powf(2.0) / 2.0)))
    }

    // Black-Scholes d2
    fn get_d2(&self, d1: &f64, sigma: &f64, duration: &f64) -> f64 {
        d1 - sigma * duration.sqrt()
    }

    // Black-Scholes-Merton option price
    fn bsm_price(&self) -> Vec<f64> {
        fn get_prices(
            opt_type: &OptTypes,
            d1: &f64,
            d2: &f64,
            underlying: &f64,
            dividend: &f64,
            duration: &f64,
            strike: &f64,
            rfr: &f64,
        ) -> f64 {
            // Standard Normal struct
            let n = Normal::new(0.0, 1.0).unwrap();

            if *opt_type == OptTypes::Call {
                underlying * (-dividend * duration).exp() * n.cdf(*d1) - strike * (-rfr * duration).exp() * n.cdf(*d2)
            } else if *opt_type == OptTypes::Put {
                strike * (-rfr * duration).exp() * n.cdf(-*d2) - underlying * (-dividend * duration).exp() * n.cdf(-*d1)
            } else { panic!("This should not be able to happen, option type outside enum.") }
        }

        // Initialize vectors
        let mut d1: Vec<f64> = Vec::with_capacity(self.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(self.tickers.len());
        let mut prices = Vec::with_capacity(self.tickers.len());

        // Compute prices
        for i in 0..self.tickers.len() {
            d1.push(self.get_d1(
                &self.underlying[i],
                &self.strike[i],
                &self.dividend[i],
                &self.rfr[i],
                &self.sigma[i],
                &self.duration[i],
            ));
            d2.push(self.get_d2(&d1[i], &self.sigma[i], &self.duration[i]));
            prices.push(get_prices(
                &self.opt_types[i],
                &d1[i],
                &d2[i],
                &self.underlying[i],
                &self.dividend[i],
                &self.duration[i],
                &self.strike[i],
                &self.rfr[i],
            ))
        }
        prices
    }

    // todo!(Alexander): Comment this section
    pub fn init_greeks(&self) -> Vec<Greeks> {
        fn get_delta(n: &Normal, opt_type: &OptTypes, d1: &f64, dividend: &f64, duration: &f64) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (-(dividend * duration)).exp() * n.cdf(*d1)
            } else if *opt_type == OptTypes::Put {
                ret = (-(dividend * duration)).exp() * (n.cdf(*d1) - 1.0)
            }
            ret
        }

        fn get_gamma(n: &Normal, d1: &f64, dividend: &f64, duration: &f64, underlying: &f64, sigma: &f64) -> f64 {
            (((-(dividend * duration)).exp()) / (underlying * sigma * duration.sqrt())) * n.pdf(*d1)
        }

        fn get_vega(n: &Normal, d1: &f64, underlying: &f64, dividend: &f64, duration: &f64) -> f64 {
            (1.0 / 100.0) * underlying * (-(dividend * duration)).exp() * duration.sqrt() * n.pdf(*d1)
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
                    * (-(((underlying * sigma * (-(dividend * duration)).exp()) / 2.0 * duration.sqrt()) * n.pdf(*d1))
                    - rfr * strike * (-(rfr * duration)).exp() * n.cdf(*d2)
                    + dividend * underlying * (-(dividend * duration)).exp() * n.cdf(*d1))
            } else if *opt_type == OptTypes::Put {
                ret = (1.0 / 365.25)
                    * (-(((underlying * sigma * (-(dividend * duration)).exp()) / 2.0 * duration.sqrt()) * n.pdf(*d1))
                    + rfr * strike * (-(rfr * duration)).exp() * n.cdf(-*d2)
                    - dividend * underlying * (-(dividend * duration)).exp() * n.cdf(-*d1))
            }
            ret
        }

        fn get_rho(n: &Normal, opt_type: &OptTypes, d2: &f64, strike: &f64, duration: &f64, rfr: &f64) -> f64 {
            let mut ret = 0.0;
            if *opt_type == OptTypes::Call {
                ret = (1.0 / 100.0) * strike * duration * (-(rfr * duration)).exp() * n.cdf(*d2)
            } else if *opt_type == OptTypes::Put {
                ret = (-1.0 / 100.0) * strike * duration * (-(rfr * duration)).exp() * n.cdf(-*d2)
            }
            ret
        }

        let n = Normal::new(0.0, 1.0).unwrap();
        let mut gr = Vec::with_capacity(self.tickers.len());

        let mut d1: Vec<f64> = Vec::with_capacity(self.tickers.len());
        let mut d2: Vec<f64> = Vec::with_capacity(self.tickers.len());

        for i in 0..self.tickers.len() {
            d1.push(self.get_d1(
                &self.underlying[i],
                &self.strike[i],
                &self.dividend[i],
                &self.rfr[i],
                &self.sigma[i],
                &self.duration[i],
            ));
            d2.push(self.get_d2(&d1[i], &self.sigma[i], &self.duration[i]));
        }

        for i in 0..self.tickers.len() {
            gr.push(Greeks {
                delta: get_delta(&n, &self.opt_types[i], &d1[i], &self.dividend[i], &self.duration[i]),
                gamma: get_gamma(&n, &d1[i], &self.dividend[i], &self.duration[i], &self.underlying[i], &self.sigma[i]),
                vega: get_vega(&n, &d1[i], &self.underlying[i], &self.dividend[i], &self.duration[i]),
                theta: get_theta(
                    &self.opt_types[i],
                    &n,
                    &d1[i],
                    &d2[i],
                    &self.underlying[i],
                    &self.dividend[i],
                    &self.duration[i],
                    &self.strike[i],
                    &self.rfr[i],
                    &self.sigma[i],
                ),
                rho: get_rho(
                    &n,
                    &self.opt_types[i],
                    &d2[i],
                    &self.strike[i],
                    &self.duration[i],
                    &self.rfr[i],
                ),
            })
        }
        gr
    }
}

fn get_durs(settles: &[DateTime<Utc>], maturities: &[DateTime<Utc>]) -> Vec<f64> {
    let mut durs: Vec<f64> = Vec::with_capacity(settles.len());
    for i in 0..settles.len() {
        durs.push((maturities[i] - settles[i]).num_seconds() as f64 / SEC_YEAR)
    }
    durs
}

fn parse_date(s: &str) -> String {
    let mut s_ret;

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
        Some(i) => s_ret = format!("{}{}{}{}", s.to_string().get(0..*i).unwrap(), "t", s.to_string().get(*i + 1..*i + 9).unwrap(), "+00:00"),
        None => {}
    }
    s_ret
}

fn file_iter(path: &Path) -> BufReader<File> {
    let file: BufReader<File>;
    loop {
        if let Ok(f) = File::open(&path) {
            file = BufReader::new(f);
            break;
        }
    }
    file
}

pub fn parse_input(
    path: PathBuf,
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
    let mut file;
    let mut lines_num;
    loop {
        file = file_iter(&path);
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

    // initializing Vectors
    let mut tickers: Vec<String> = Vec::with_capacity(lines_num);
    let mut opt_types: Vec<OptTypes> = Vec::with_capacity(lines_num);
    let mut underlying: Vec<f64> = Vec::with_capacity(lines_num);
    let mut strike: Vec<f64> = Vec::with_capacity(lines_num);
    let mut settles: Vec<DateTime<Utc>> = Vec::with_capacity(lines_num);
    let mut maturities: Vec<DateTime<Utc>> = Vec::with_capacity(lines_num);
    let mut dividend: Vec<f64> = Vec::with_capacity(lines_num);
    let mut rfr: Vec<f64> = Vec::with_capacity(lines_num);
    let mut sigma: Vec<f64> = Vec::with_capacity(lines_num);

    if let Ok(mut lines) = read_lines(&path) {
        let procc_s = lines.next().unwrap().unwrap();
        let headers: Vec<&str> = procc_s.split(',').collect();
        let tick_idx = headers.iter().position(|x| x.to_lowercase() == "ticker").expect("No header tickers in file");
        let opt_t_idx = headers.iter().position(|x| x.to_lowercase() == "opt_type").expect("No header opt_type in file");
        let underlying_idx = headers.iter().position(|x| x.to_lowercase() == "underlying").expect("No header underlying in file");
        let strike_idx = headers.iter().position(|x| x.to_lowercase() == "strike").expect("No header strike in file");
        let set_idx = headers.iter().position(|x| x.to_lowercase() == "settle").expect("No header settle in file");
        let mat_idx = headers.iter().position(|x| x.to_lowercase() == "maturity").expect("No header maturity in file");
        let dividend_idx = headers.iter().position(|x| x.to_lowercase() == "dividend").expect("No header dividend in file");
        let rfr_idx = headers.iter().position(|x| x.to_lowercase() == "rfr").expect("No header rfr in file");
        let sigma_idx = headers.iter().position(|x| x.to_lowercase() == "sigma").expect("No header sigma in file");

        // push data
        for line in lines.flatten() {
            let inps: Vec<&str> = line.split(',').collect();
            tickers.push(inps[tick_idx].to_string());
            opt_types.push(OptTypes::from_str(inps[opt_t_idx]).unwrap());
            underlying.push(inps[underlying_idx].parse::<f64>().expect("failed to parse s to f64"));
            strike.push(inps[strike_idx].parse::<f64>().expect("failed to parse k to f64"));
            settles.push(DateTime::from(
                DateTime::parse_from_rfc3339(&*parse_date(inps[set_idx])).unwrap(),
            ));
            maturities.push(DateTime::from(
                DateTime::parse_from_rfc3339(&*parse_date(inps[mat_idx])).unwrap(),
            ));
            dividend.push(inps[dividend_idx].parse::<f64>().expect("failed to parse s to f64"));
            rfr.push(inps[rfr_idx].parse::<f64>().expect("failed to parse s to f64"));
            sigma.push(inps[sigma_idx].parse::<f64>().expect("failed to parse s to f64"));
        }
    }
    (tickers, opt_types, underlying, strike, settles, maturities, dividend, rfr, sigma)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<BufReader<File>>>
    where
        P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

// todo!(Alexander): Refactor to initialize_full_opts
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
    // Initialize Options
    let mut opts = Options::new(
        tup.0, tup.1, tup.2, tup.3, tup.4, tup.5, tup.6, tup.7, tup.8,
    );

    // Initialize greeks
    opts.greeks = opts.init_greeks();

    // Return Options with greeks
    opts
}

// Writing Options struct to csv
// Note: input opts: Vec<Options>. Reason: Rayon has chunked data for computation into Vec<Options> in options-listener
pub fn write_csv_out(path: PathBuf, opts: Vec<Options>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    // Column headers
    let headers = [
        "ticker", "opt_type", "underlying", "strike", "settle", "maturity", "duration", "dividend", "rfr", "sigma", "price", "delta",
        "gamma", "vega", "theta", "rho",
    ];
    wtr.write_record(headers).expect("failed writing headers");

    // Collects chunked options back into one file, see function comment
    for chunk in opts {
        for rec in chunk {
            wtr.write_record(rec)
                .expect("failed writing iterated result.");
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn test_prices() {}
