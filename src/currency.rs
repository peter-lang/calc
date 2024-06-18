use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::sync::OnceLock;

use chrono::{Days, NaiveDate, Utc};
use quick_xml::de;
use serde::{Deserialize, Deserializer};

use crate::error::CalcError;
use crate::files;
use crate::rational::Rational;

#[derive(Deserialize)]
struct GetCurrentExchangeRatesResponse {
    #[serde(rename = "GetCurrentExchangeRatesResult")]
    get_current_exchange_rates_result: String,
}

#[derive(Deserialize)]
struct Body {
    #[serde(rename = "GetCurrentExchangeRatesResponse")]
    get_current_exchange_rates_response: GetCurrentExchangeRatesResponse,
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "Body")]
    body: Body,
}

fn deserialize_rate<'de, D>(deserializer: D) -> Result<Rational, D::Error>
    where
        D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let trimmed = buf.trim_end_matches('0');
    if let Some(idx) = trimmed.rfind(',') {
        let decimals = (trimmed.len() - idx - 1) as u32;
        let num = trimmed
            .replace(',', "")
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Rational::new(num, 10_u64.pow(decimals)))
    } else {
        let num = trimmed.parse().map_err(serde::de::Error::custom)?;
        Ok(Rational { num, den: 1 })
    }
}

#[derive(Deserialize)]
struct MNBCurrentExchangeRate {
    #[serde(rename = "@curr")]
    curr: String,
    #[serde(rename = "@unit")]
    unit: u32,
    #[serde(rename = "$value", deserialize_with = "deserialize_rate")]
    rate: Rational,
}

#[derive(Deserialize)]
struct MNBCurrentExchangeRateDay {
    #[serde(rename = "@date")]
    date: NaiveDate,
    #[serde(rename = "Rate")]
    rates: Vec<MNBCurrentExchangeRate>,
}

#[derive(Deserialize)]
struct MNBCurrentExchangeRates {
    #[serde(rename = "Day")]
    day: MNBCurrentExchangeRateDay,
}

fn fetch_current_rate_xml() -> Result<String, CalcError> {
    const MNB_BODY: &str = r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:web="http://www.mnb.hu/webservices/"><soapenv:Header/><soapenv:Body><web:GetCurrentExchangeRates/></soapenv:Body></soapenv:Envelope>"#;
    let response = ureq::post("http://www.mnb.hu/arfolyamok.asmx")
        .set("Content-Type", "text/xml;charset=UTF-8")
        .send_bytes(MNB_BODY.as_bytes())?;
    let result: Envelope = de::from_reader(BufReader::new(response.into_reader()))?;
    Ok(result
        .body
        .get_current_exchange_rates_response
        .get_current_exchange_rates_result)
}

const RATE_FILE_NAME: &str = "rates.xml";

fn save_current_rate_xml_file(xml: &str) -> () {
    let file = files::cache(RATE_FILE_NAME);
    let _ = fs::write(file, xml);
}

fn read_current_rate_xml_file() -> Option<String> {
    let file = files::cache(RATE_FILE_NAME);
    if !file.exists() {
        return None;
    }
    fs::read_to_string(file).ok()
}

fn to_map(rates: Vec<MNBCurrentExchangeRate>) -> HashMap<String, Rational> {
    rates
        .into_iter()
        .map(|x| {
            if x.unit == 1 {
                (x.curr.to_ascii_uppercase(), x.rate)
            } else {
                (
                    x.curr.to_ascii_uppercase(),
                    x.rate / Rational::new(x.unit as i64, 1),
                )
            }
        })
        .collect()
}

pub fn convert(from: &str, to: &str) -> Result<Rational, CalcError> {
    static RATES: OnceLock<Result<HashMap<String, Rational>, CalcError>> = OnceLock::new();
    let map = RATES
        .get_or_init(|| {
            let today = Utc::now().date_naive();
            let yesterday = today - Days::new(1);
            if let Some(content) = read_current_rate_xml_file() {
                let rates: MNBCurrentExchangeRates = de::from_str(content.as_str())?;
                if rates.day.date == today || rates.day.date == yesterday {
                    return Ok(to_map(rates.day.rates));
                }
            }
            let content = fetch_current_rate_xml()?;
            save_current_rate_xml_file(content.as_str());
            let rates: MNBCurrentExchangeRates = de::from_str(content.as_str())?;
            return Ok(to_map(rates.day.rates));
        })
        .as_ref()?;

    const BASE_CURRENCY: &'static str = "HUF";
    return if from == BASE_CURRENCY {
        let Some(inv_rate) = map.get(to) else {
            return Err(CalcError::ConversionError);
        };
        Ok(inv_rate.invert()?)
    } else if to == BASE_CURRENCY {
        let Some(rate) = map.get(from) else {
            return Err(CalcError::ConversionError);
        };
        Ok(rate.clone())
    } else {
        let Some(inv_rate) = map.get(to) else {
            return Err(CalcError::ConversionError);
        };
        let Some(rate) = map.get(from) else {
            return Err(CalcError::ConversionError);
        };
        Ok(inv_rate.invert()? * rate.clone())
    };
}
