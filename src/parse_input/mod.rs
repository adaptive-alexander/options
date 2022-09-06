use super::OptTypes;
use chrono::offset::Utc;
use chrono::DateTime;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
        let sigma_idx = headers
            .iter()
            .position(|x| x.to_lowercase() == "sigma")
            .expect("No header sigma in file");

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
            sigma.push(
                inps[sigma_idx]
                    .parse::<f64>()
                    .expect("failed to parse s to f64"),
            );
        }
    }
    (
        tickers, opt_types, underlying, strike, settles, maturities, dividend, rfr, sigma,
    )
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}
