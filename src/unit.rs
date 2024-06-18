use crate::{currency, number_op};
use crate::error::CalcError;
use crate::number::Number;
use crate::rational::Rational;

#[derive(PartialEq)]
pub enum UnitType {
    Length,
    Area,
    Volume,
    Mass,
    Temperature,
    Time,
    Currency,
}

#[derive(Clone, PartialEq)]
pub enum Unit {
    Curr(&'static str), // currency

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
    VolMilliLiter, // ml
    VolM,          // m3
    VolCm,         // cm3
    VolMm,         // mm3
    VolInch,       // in3
    VolFeet,       // ft3
    VolYard,       // yd3
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
}

const fn wrap(rate: Rational) -> Result<Number, CalcError> {
    if rate.den == 1 {
        Ok(Number::Int(rate.num))
    } else {
        Ok(Number::Rational(rate))
    }
}

pub fn get_default_factor(a: &Unit) -> Result<Number, CalcError> {
    const _1: Rational = Rational::new(1, 1);
    const _1000: Rational = Rational::new(1000, 1);
    const _1_1000: Rational = Rational::new(1, 1000);
    const _1_100: Rational = Rational::new(1, 100);
    const INCH2M: Rational = Rational::new(254, 10000);
    const FEET2INCH: Rational = Rational::new(12, 1);
    const YARD2INCH: Rational = Rational::new(36, 1);
    const MILE2INCH: Rational = Rational::new(63360, 1);
    match a {
        Unit::LenM => wrap(_1),
        Unit::LenKm => wrap(_1000),
        Unit::LenCm => wrap(_1_100),
        Unit::LenMm => wrap(_1_1000),
        Unit::LenInch => wrap(INCH2M),
        Unit::LenFeet => wrap(FEET2INCH * INCH2M),
        Unit::LenYard => wrap(YARD2INCH * INCH2M),
        Unit::LenMile => wrap(MILE2INCH * INCH2M),
        Unit::AreaM => wrap(_1),
        Unit::AreaKm => wrap(_1000.pow(2)),
        Unit::AreaCm => wrap(_1_100.pow(2)),
        Unit::AreaMm => wrap(_1_1000.pow(2)),
        Unit::AreaInch => wrap(INCH2M.pow(2)),
        Unit::AreaFeet => wrap((FEET2INCH * INCH2M).pow(2)),
        Unit::AreaYard => wrap((YARD2INCH * INCH2M).pow(2)),
        Unit::AreaMile => wrap((MILE2INCH * INCH2M).pow(2)),
        Unit::VolM => wrap(_1000),
        Unit::VolCm => wrap(_1_100.pow(3) * _1000),
        Unit::VolMm => wrap(_1_1000.pow(3) * _1000),
        Unit::VolInch => wrap(INCH2M.pow(3) * _1000),
        Unit::VolFeet => wrap((FEET2INCH * INCH2M).pow(3) * _1000),
        Unit::VolYard => wrap((YARD2INCH * INCH2M).pow(3) * _1000),
        Unit::VolMilliLiter => wrap(_1_1000),
        Unit::VolLiter => wrap(_1),
        Unit::VolPint => wrap(Rational::new(454609, 800000)),
        Unit::VolGallon => wrap(Rational::new(454609, 100000)),
        Unit::MassG => wrap(_1_1000),
        Unit::MassKg => wrap(_1),
        Unit::MassOunce => wrap(Rational::new(45359237, 800000000)),
        Unit::MassPound => wrap(Rational::new(45359237, 100000000)),
        Unit::TimeSec => wrap(_1),
        Unit::TimeMin => wrap(Rational::new(60, 1)),
        Unit::TimeHour => wrap(Rational::new(3600, 1)),
        _ => {
            return Err(CalcError::ConversionError);
        }
    }
}

pub fn get_unit_name(a: &Unit) -> &str {
    match a {
        Unit::Curr(x) => x,
        Unit::LenM => "m",
        Unit::LenKm => "km",
        Unit::LenCm => "cm",
        Unit::LenMm => "mm",
        Unit::LenInch => "\"",
        Unit::LenFeet => "'",
        Unit::LenYard => "yd",
        Unit::LenMile => "mi",
        Unit::AreaM => "m2",
        Unit::AreaKm => "km2",
        Unit::AreaCm => "cm2",
        Unit::AreaMm => "mm2",
        Unit::AreaInch => "in2",
        Unit::AreaFeet => "ft2",
        Unit::AreaYard => "yd2",
        Unit::AreaMile => "mi2",
        Unit::VolLiter => "l",
        Unit::VolMilliLiter => "ml",
        Unit::VolM => "m3",
        Unit::VolCm => "cm3",
        Unit::VolMm => "mm3",
        Unit::VolInch => "in3",
        Unit::VolFeet => "ft3",
        Unit::VolYard => "yd3",
        Unit::VolPint => "pint",
        Unit::VolGallon => "gallon",
        Unit::MassG => "g",
        Unit::MassKg => "kg",
        Unit::MassOunce => "oz",
        Unit::MassPound => "lb",
        Unit::TempC => "C",
        Unit::TempF => "F",
        Unit::TimeSec => "s",
        Unit::TimeMin => "min",
        Unit::TimeHour => "h",
    }
}

pub fn get_unit_type(a: &Unit) -> UnitType {
    match a {
        Unit::Curr(_) => UnitType::Currency,
        Unit::LenM => UnitType::Length,
        Unit::LenKm => UnitType::Length,
        Unit::LenCm => UnitType::Length,
        Unit::LenMm => UnitType::Length,
        Unit::LenInch => UnitType::Length,
        Unit::LenFeet => UnitType::Length,
        Unit::LenYard => UnitType::Length,
        Unit::LenMile => UnitType::Length,
        Unit::AreaM => UnitType::Area,
        Unit::AreaKm => UnitType::Area,
        Unit::AreaCm => UnitType::Area,
        Unit::AreaMm => UnitType::Area,
        Unit::AreaInch => UnitType::Area,
        Unit::AreaFeet => UnitType::Area,
        Unit::AreaYard => UnitType::Area,
        Unit::AreaMile => UnitType::Area,
        Unit::VolLiter => UnitType::Volume,
        Unit::VolMilliLiter => UnitType::Volume,
        Unit::VolM => UnitType::Volume,
        Unit::VolCm => UnitType::Volume,
        Unit::VolMm => UnitType::Volume,
        Unit::VolInch => UnitType::Volume,
        Unit::VolFeet => UnitType::Volume,
        Unit::VolYard => UnitType::Volume,
        Unit::VolPint => UnitType::Volume,
        Unit::VolGallon => UnitType::Volume,
        Unit::MassG => UnitType::Mass,
        Unit::MassKg => UnitType::Mass,
        Unit::MassOunce => UnitType::Mass,
        Unit::MassPound => UnitType::Mass,
        Unit::TempC => UnitType::Temperature,
        Unit::TempF => UnitType::Temperature,
        Unit::TimeSec => UnitType::Time,
        Unit::TimeMin => UnitType::Time,
        Unit::TimeHour => UnitType::Time,
    }
}

fn convert_temp(val: Number, from: &Unit, to: &Unit) -> Result<Number, CalcError> {
    const FACTOR: Number = Number::Rational(Rational::new(18, 10));
    const BIAS: Number = Number::Int(32);

    match (from, to) {
        (Unit::TempC, Unit::TempF) => return Ok(number_op::add(number_op::mul(val, FACTOR), BIAS)),
        (Unit::TempF, Unit::TempC) => {
            return Ok(number_op::div(number_op::sub(val, BIAS), FACTOR)?)
        }
        (_, _) => Err(CalcError::ConversionError),
    }
}

pub fn common_type(a: &Unit, b: &Unit) -> Option<UnitType> {
    let unit_type = get_unit_type(a);
    if unit_type == get_unit_type(b) {
        return Some(unit_type);
    } else {
        None
    }
}

pub fn convert(val: Number, from: &Unit, to: &Unit) -> Result<Number, CalcError> {
    let Some(unit_type) = common_type(from, to) else {
        return Err(CalcError::DifferentUnitTypes);
    };
    match unit_type {
        UnitType::Temperature => convert_temp(val, from, to),
        UnitType::Currency => {
            let Unit::Curr(from) = from else {
                return Err(CalcError::ConversionError);
            };
            let Unit::Curr(to) = to else {
                return Err(CalcError::ConversionError);
            };
            return Ok(number_op::mul(
                val,
                Number::Rational(currency::convert(*from, *to)?),
            ));
        }
        _ => {
            let rate = number_op::div(get_default_factor(from)?, get_default_factor(to)?)?;
            Ok(number_op::mul(rate, val))
        }
    }
}

pub fn single(a: Option<Unit>, b: Option<Unit>) -> Result<Option<Unit>, CalcError> {
    let Some(a) = a else {
        return Ok(b);
    };
    let Some(_) = b else {
        return Ok(Some(a));
    };
    Err(CalcError::OperateWithUnits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_pint_to_gallon() {
        let result = convert(Number::Int(1), &Unit::VolPint, &Unit::VolGallon);
        assert_eq!(result, Ok(Number::Rational(Rational::new(1, 8))));
    }

    #[test]
    fn test_convert_fahrenheit_to_celsius() {
        let result = convert(Number::Int(95), &Unit::TempF, &Unit::TempC);
        assert_eq!(result, Ok(Number::Int(35)));
    }

    #[test]
    fn test_convert_celsius_to_fahrenheit() {
        let result = convert(Number::Int(15), &Unit::TempC, &Unit::TempF);
        assert_eq!(result, Ok(Number::Int(59)));
    }
}
