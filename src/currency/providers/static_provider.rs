use std::collections::HashMap;

use crate::currency::RateProvider;
use crate::error::CalcError;
use crate::rational::Rational;

pub struct StaticProvider {
    rates: HashMap<String, Rational>,
}

impl StaticProvider {
    pub fn new(raw: &HashMap<String, f64>) -> Self {
        let rates = raw
            .iter()
            .map(|(k, v)| (k.clone(), f64_to_rational(*v)))
            .collect();
        Self { rates }
    }
}

impl RateProvider for StaticProvider {
    fn id(&self) -> &str {
        "static"
    }

    fn convert(&self, from: &str, to: &str) -> Result<Rational, CalcError> {
        let key = format!("{}/{}", from, to);
        self.rates.get(&key).cloned().ok_or(CalcError::ConversionError)
    }
}

fn f64_to_rational(x: f64) -> Rational {
    let s = format!("{}", x);
    if let Some(dot) = s.find('.') {
        let decimals = (s.len() - dot - 1) as u32;
        let num: i64 = s.replace('.', "").parse().unwrap_or(0);
        Rational::new(num, 10_u64.pow(decimals))
    } else {
        Rational::new(x as i64, 1)
    }
}
