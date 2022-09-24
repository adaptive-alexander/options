use crate::opt_data::OptData;
use crate::options_struct::Options;
use crate::pricing_models::black_scholes::BlackScholesModel;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// # chunk_opts
/// Chunk a single large [`Options`] into chunks for parallel computation.
///
/// # args:
/// * `opt` - Input option to chunk
/// * `size` - Size of chunks (1000 is a good starting point)
///
/// # returns:
/// A Vec of [`Options`]. Maximum size of each `Options` set by size argument.
pub fn chunk_opt(opt: Options, size: usize) -> Vec<Options> {
    let n_options = opt.opt_data.tickers.len(); // Number of options
    let chunks = (n_options as f64 / size as f64) as usize; // Number of chunks
    let remaining = n_options % size;
    let mut chunk_vec = Vec::with_capacity(chunks);
    let mut idx;
    for i in 0..=(chunks - 1) {
        idx = i * size; // Starting index of next chunk
        // If there are full chunks left to allocate
        chunk_vec.push(Options::new(
            OptData::new(
                opt.opt_data.tickers[idx..idx + size].to_vec(),
                opt.opt_data.opt_types[idx..idx + size].to_vec(),
                opt.opt_data.underlying[idx..idx + size].to_vec(),
                opt.opt_data.strike[idx..idx + size].to_vec(),
                opt.opt_data.settles[idx..idx + size].to_vec(),
                opt.opt_data.maturities[idx..idx + size].to_vec(),
                opt.opt_data.dividend[idx..idx + size].to_vec(),
                opt.opt_data.rfr[idx..idx + size].to_vec(),
                opt.opt_data.volatility[idx..idx + size].to_vec(),
            ),
            Box::new(BlackScholesModel::new()),
        ));
        println!("From {} to {}", idx, idx + size);
    };
    if remaining != 0 {
        // If not enough for full chunk allocate the rest
        chunk_vec.push(Options::new(
            OptData::new(
                opt.opt_data.tickers[idx..n_options].to_vec(),
                opt.opt_data.opt_types[idx..n_options].to_vec(),
                opt.opt_data.underlying[idx..n_options].to_vec(),
                opt.opt_data.strike[idx..n_options].to_vec(),
                opt.opt_data.settles[idx..n_options].to_vec(),
                opt.opt_data.maturities[idx..n_options].to_vec(),
                opt.opt_data.dividend[idx..n_options].to_vec(),
                opt.opt_data.rfr[idx..n_options].to_vec(),
                opt.opt_data.volatility[idx..n_options].to_vec(),
            ),
            Box::new(BlackScholesModel::new()),
        ))
    }

    chunk_vec
}

/// # collect_chunks
/// Takes a Vec of options and returns a single [`Options`]
///
/// # args:
/// *`opts` - A vector of options
///
/// # returns:
/// A single [`Options`] containing the combined data
pub fn collect_chunks(opts: Vec<Options>) -> Options {
    let mut ret_opt = Options::default();
    for opt in opts {
        ret_opt.opt_data.tickers.extend(opt.opt_data.tickers);
        ret_opt.opt_data.opt_types.extend(opt.opt_data.opt_types);
        ret_opt.opt_data.underlying.extend(opt.opt_data.underlying);
        ret_opt.opt_data.strike.extend(opt.opt_data.strike);
        ret_opt.opt_data.settles.extend(opt.opt_data.settles);
        ret_opt.opt_data.maturities.extend(opt.opt_data.maturities);
        ret_opt.opt_data.duration.extend(opt.opt_data.duration);
        ret_opt.opt_data.dividend.extend(opt.opt_data.dividend);
        ret_opt.opt_data.rfr.extend(opt.opt_data.rfr);
        ret_opt.opt_data.volatility.extend(opt.opt_data.volatility);
        ret_opt.prices.extend(opt.prices);
        ret_opt.greeks.extend(opt.greeks);
    }
    ret_opt
}

/// # retry_open_file
/// Retries opening a file until successful.
///
/// # args:
/// *`path` - Path to file to open.
pub fn retry_open_file(path: &PathBuf) -> BufReader<File> {
    let file: BufReader<File>;
    loop {
        if let Ok(f) = File::open(&path) {
            file = BufReader::new(f);
            break;
        }
    }
    file
}
