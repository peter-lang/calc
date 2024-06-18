use regex::{Captures, Regex};

use super::token::{CURRENCIES_PATTERN, Token};

pub struct Lexer {
    patterns: Regex,
}

static PATTERNS: [(&'static str, fn(&str) -> Token); Token::VARIANT_COUNT] = [
    (
        r"(?:(?:[0-9]*\.[0-9]+)|(?:[0-9]+\.))(?:[eE][-+]?[0-9]+)?",
        |x| Token::LitFloat(x.parse::<f64>().unwrap()),
    ),
    (r"[0-9]+(?:[eE][-+]?[0-9]+)?", |x| {
        Token::LitInt(x.parse::<i64>().unwrap())
    }),
    (r"\(", |_| Token::ParBegin),
    (r"\)", |_| Token::ParEnd),
    (r"\^|\*\*", |_| Token::Exp),
    (r"\-", |_| Token::Sub),
    (r"\+", |_| Token::Add),
    (r"\*", |_| Token::Mul),
    ("/", |_| Token::Div),
    ("%", |_| Token::Mod),
    ("to", |_| Token::KwTo),
    (CURRENCIES_PATTERN, |x| {
        Token::Curr(String::from(x.to_ascii_lowercase()))
    }),
    // 3 char
    ("cm3", |_| Token::VolCm),
    ("mm3", |_| Token::VolMm),
    ("in3", |_| Token::VolInch),
    ("ft3", |_| Token::VolFeet),
    ("yd3", |_| Token::VolYard),
    ("gallon|gal", |_| Token::VolGallon),
    ("km2", |_| Token::AreaKm),
    ("cm2", |_| Token::AreaCm),
    ("mm2", |_| Token::AreaMm),
    ("in2", |_| Token::AreaInch),
    ("ft2", |_| Token::AreaFeet),
    ("yd2", |_| Token::AreaYard),
    ("mi2", |_| Token::AreaMile),
    ("min", |_| Token::TimeMin),
    // 2 char
    ("pint|pt", |_| Token::VolPint),
    ("ml", |_| Token::VolMilliLiter),
    ("km", |_| Token::LenKm),
    ("cm", |_| Token::LenCm),
    ("mm", |_| Token::LenMm),
    ("inch|in|\"", |_| Token::LenInch),
    ("feet|ft|'", |_| Token::LenFeet),
    ("yard|yd", |_| Token::LenYard),
    ("mi", |_| Token::LenMile),
    ("m2", |_| Token::AreaM),
    ("m3", |_| Token::VolM),
    ("kg", |_| Token::MassKg),
    ("ounce|oz", |_| Token::MassOunce),
    ("pound|lb", |_| Token::MassPound),
    // 1 char
    ("sec|s", |_| Token::TimeSec),
    ("hour|hr|h", |_| Token::TimeHour),
    ("m", |_| Token::LenM),
    ("C|c", |_| Token::TempC),
    ("F|f", |_| Token::TempF),
    ("g", |_| Token::MassG),
    ("liter|l", |_| Token::VolLiter),
    ("[A-Za-z_][A-Za-z0-9_]*", |x| Token::Ident(String::from(x))),
    (r"\S+", |x| Token::INVALID(String::from(x))),
];

impl Lexer {
    pub fn new() -> Self {
        let pattern = PATTERNS
            .iter()
            .map(|(pat, _)| format!("({pat})"))
            .collect::<Vec<String>>()
            .join("|");
        let pattern = Regex::new(pattern.as_str()).unwrap();

        Lexer { patterns: pattern }
    }

    fn map_captures(captures: Captures) -> Token {
        for idx in 1..=Token::VARIANT_COUNT {
            if let Some(m) = captures.get(idx) {
                let found = m.as_str();
                let (_, extract) = PATTERNS[idx - 1];
                let token = extract(found);
                return token;
            }
        }
        return Token::INVALID(String::from(""));
    }

    pub fn parse<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Token> + 'a {
        self.patterns.captures_iter(text).map(Lexer::map_captures)
    }
}
