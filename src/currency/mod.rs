mod providers;

use std::sync::OnceLock;

use crate::config::{self, CurrencyProvider};
use crate::error::CalcError;
use crate::rational::Rational;

use providers::mnb::MnbProvider;
use providers::static_provider::StaticProvider;

pub trait RateProvider: Sync {
    #[allow(dead_code)]
    fn id(&self) -> &str;
    fn convert(&self, from: &str, to: &str) -> Result<Rational, CalcError>;
}

fn active_provider() -> &'static dyn RateProvider {
    static MNB: MnbProvider = MnbProvider;
    static STATIC: OnceLock<StaticProvider> = OnceLock::new();

    let guard = config::current();
    match guard.currency.provider {
        CurrencyProvider::Mnb => &MNB,
        CurrencyProvider::Static => {
            STATIC.get_or_init(|| StaticProvider::new(&guard.currency.static_rates))
        }
    }
}

pub fn convert(from: &str, to: &str) -> Result<Rational, CalcError> {
    active_provider().convert(from, to)
}
