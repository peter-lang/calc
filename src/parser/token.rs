use variant_count::VariantCount;

#[derive(PartialEq, VariantCount)]
pub enum Token {
    ParBegin, // (
    ParEnd,   // )
    Exp,      // ^, **
    Sub,      // -
    Add,      // +
    Mul,      // *
    Div,      // /
    Mod,      // %
    KwTo,     // to

    LitFloat(f64), // float
    LitInt(i64),   // int

    Curr(String), // currency

    LenM,    // m
    LenKm,   // km
    LenCm,   // cm
    LenMm,   // mm
    LenInch, // in, inch, "
    LenFeet, // ft, feet, '
    LenYard, // yd, yard
    LenMile, // mi

    AreaM,    // m2
    AreaKm,   // km2
    AreaCm,   // cm2
    AreaMm,   // mm2
    AreaInch, // in2
    AreaFeet, // ft2
    AreaYard, // yd2
    AreaMile, // mi2

    VolLiter,      // l, liter
    VolM,          // m3
    VolCm,         // cm3
    VolMm,         // mm3
    VolInch,       // in3
    VolFeet,       // ft3
    VolYard,       // yd3
    VolMilliLiter, // ml
    VolPint,       // pt, pint
    VolGallon,     // gal, gallon

    MassG,     // g
    MassKg,    // kg
    MassOunce, // ounce, oz
    MassPound, // pound, lb

    TempC, // c
    TempF, // f
    //
    TimeSec,  // s, sec
    TimeMin,  // min
    TimeHour, // hour, hr

    Ident(String),

    INVALID(String),
}

pub const CURRENCIES: [&'static str; 34] = [
    "aud", "bgn", "brl", "cad", "chf", "cny", "czk", "dkk", "eur", "gbp", "hkd", "huf", "idr",
    "ils", "inr", "isk", "jpy", "krw", "mxn", "myr", "nok", "nzd", "php", "pln", "ron", "rsd",
    "rub", "sek", "sgd", "thb", "try", "uah", "usd", "zar",
];

// TODO: this should come from a macro
pub const CURRENCIES_PATTERN: &'static str = "AUD|aud|BGN|bgn|BRL|brl|CAD|cad|CHF|chf|CNY|cny|CZK|czk|DKK|dkk|EUR|eur|GBP|gbp|HKD|hkd|HUF|huf|IDR|idr|ILS|ils|INR|inr|ISK|isk|JPY|jpy|KRW|krw|MXN|mxn|MYR|myr|NOK|nok|NZD|nzd|PHP|php|PLN|pln|RON|ron|RSD|rsd|RUB|rub|SEK|sek|SGD|sgd|THB|thb|TRY|try|UAH|uah|USD|usd|ZAR|zar";
