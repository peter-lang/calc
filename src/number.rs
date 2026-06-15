use std::fmt::{Display, Formatter};

use crate::config::{self, FormatOptions, NumberRepr};
use crate::rational::Rational;

#[derive(Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Rational(Rational),
    Float(f64),
}

pub fn format_number(num: &Number, opts: &FormatOptions) -> String {
    match num {
        Number::Int(x) => format_int(*x, opts),
        Number::Rational(r) => match opts.repr {
            NumberRepr::Rational => format!("{}/{}", r.num, r.den),
            _ => {
                let f: f64 = r.into();
                format_float(f, opts)
            }
        },
        Number::Float(x) => format_float(*x, opts),
    }
}

fn format_int(x: i64, opts: &FormatOptions) -> String {
    match opts.repr {
        NumberRepr::Financial => format_financial(x as f64, opts.fin.precision),
        NumberRepr::Sci => format_scientific(x as f64, opts.sci.precision),
        _ => {
            if opts.int.sci_upgrade && (x.unsigned_abs() as f64) >= opts.int.sci_upgrade_upper {
                format_scientific(x as f64, opts.sci.precision)
            } else {
                format!("{}", x)
            }
        }
    }
}

fn format_float(x: f64, opts: &FormatOptions) -> String {
    if !x.is_finite() {
        return format!("{}", x);
    }
    if x.fract() == 0.0 && x.abs() <= i64::MAX as f64 {
        return format_int(x as i64, opts);
    }
    match opts.repr {
        NumberRepr::Fixed | NumberRepr::Rational => format_fixed(x, opts.float.precision),
        NumberRepr::Float => {
            let abs = x.abs();
            if opts.float.sci_upgrade
                && (abs >= opts.float.sci_upgrade_upper
                    || (abs > 0.0 && abs < opts.float.sci_upgrade_lower))
            {
                format_scientific(x, opts.sci.precision)
            } else {
                format_fixed(x, opts.float.precision)
            }
        }
        NumberRepr::Sci => format_scientific(x, opts.sci.precision),
        NumberRepr::Financial => format_financial(x, opts.fin.precision),
    }
}

fn format_financial(x: f64, precision: u8) -> String {
    if x.abs() >= 1e6 {
        format!("{}m", format_financial_fixed(x / 1e6, precision))
    } else {
        format_financial_fixed(x, precision)
    }
}

// Like format_fixed but does not trim trailing zeros (financial display preserves precision).
fn format_financial_fixed(x: f64, precision: u8) -> String {
    let p = precision as usize;
    let scale = 10f64.powi(precision as i32);
    let rounded = (x * scale).round() / scale;
    if rounded == x {
        format!("{:.prec$}", rounded, prec = p)
    } else {
        format!("{:.prec$}\u{2026}", rounded, prec = p)
    }
}

fn format_fixed(x: f64, precision: u8) -> String {
    let p = precision as usize;
    let scale = 10f64.powi(precision as i32);
    let rounded = (x * scale).round() / scale;
    if rounded == x {
        trim_zeros(format!("{:.prec$}", rounded, prec = p))
    } else {
        format!("{:.prec$}\u{2026}", rounded, prec = p)
    }
}

fn format_scientific(x: f64, precision: u8) -> String {
    let p = precision as usize;
    let raw = format!("{:.prec$e}", x, prec = p);
    let (mant_str, exp_str) = raw
        .split_once('e')
        .expect("scientific notation always has 'e'");
    let is_exact = raw.parse::<f64>().expect("valid float") == x;
    let mant_out = if is_exact {
        trim_zeros(mant_str.to_string())
    } else {
        format!("{}\u{2026}", mant_str)
    };
    format!("{}e{}", mant_out, exp_str)
}

fn trim_zeros(s: String) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_number(self, &config::current().format))
    }
}

impl From<i64> for Number {
    fn from(item: i64) -> Self {
        Number::Int(item)
    }
}

impl From<f64> for Number {
    fn from(item: f64) -> Self {
        Number::Float(item)
    }
}

impl Number {
    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(x) => x as f64,
            Number::Rational(rat) => rat.into(),
            Number::Float(x) => x,
        }
    }

    pub fn to_rational(self) -> Rational {
        match self {
            Number::Int(x) => x.into(),
            Number::Rational(rat) => rat,
            Number::Float(x) => panic!("Cannot cast float {x} to rational"),
        }
    }
}
