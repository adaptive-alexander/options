use std::fmt;

// Struct for option greeks
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
