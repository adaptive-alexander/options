#[cfg(test)]
mod test_greeks {
    use crate::greeks::Greeks;
    #[test]
    fn default() {
        let greek = Greeks::default();
        assert_eq!(greek.delta, 0.0)
    }

    #[test]
    fn display() {
        let greek = Greeks::default();
        println!("{}", greek);
    }
}

#[cfg(test)]
mod test_options {
    use crate::opt_data::OptData;
    use crate::options_struct::{OptTypes, Options};
    use crate::pricing_models::black_scholes;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn new() {
        let opt = Options::new(
            OptData::new(
                vec!["AAPL".to_string()],
                vec![OptTypes::Call],
                vec![120.0],
                vec![110.0],
                vec![chrono::DateTime::from_utc(
                    NaiveDate::from_ymd(2022, 9, 14).and_hms(2, 22, 0),
                    Utc,
                )],
                vec![chrono::DateTime::from_utc(
                    NaiveDate::from_ymd(2022, 11, 18).and_hms(15, 0, 0),
                    Utc,
                )],
                vec![0.03],
                vec![0.03],
                vec![0.35],
            ),
            Box::new(black_scholes::BlackScholesModel::new()),
        );
        assert_eq!(opt.opt_data.tickers[0], *"AAPL")
    }

    #[test]
    fn default() {
        let opt = Options::default();
        assert_eq!(opt.opt_data.tickers, Vec::<String>::new())
    }

    #[test]
    fn to_record() {
        let mut opt = Options::new(
            OptData::new(
                vec!["AAPL".to_string()],
                vec![OptTypes::Call],
                vec![120.0],
                vec![110.0],
                vec![chrono::DateTime::from_utc(
                    NaiveDate::from_ymd(2022, 9, 14).and_hms(2, 22, 0),
                    Utc,
                )],
                vec![chrono::DateTime::from_utc(
                    NaiveDate::from_ymd(2022, 11, 18).and_hms(15, 0, 0),
                    Utc,
                )],
                vec![0.03],
                vec![0.03],
                vec![0.35],
            ),
            Box::new(black_scholes::BlackScholesModel::new()),
        );
        opt.get_greeks();
        opt.get_prices();
        let records = opt.to_records();
        for rec in records {
            println!("{:?}", rec);
        }
    }
}
