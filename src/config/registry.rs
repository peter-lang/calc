use super::{Config, CurrencyProvider, NumberRepr};

/// One settable configuration value: its dotted key path, a getter/setter over
/// the live `Config`, and the candidate values offered during TAB completion
/// (empty for free-form numeric fields).
pub struct ConfigEntry {
    pub key: &'static str,
    pub get: fn(&Config) -> String,
    pub set: fn(&mut Config, &str) -> Result<(), String>,
    pub completions: &'static [&'static str],
}

fn parse_number_repr(s: &str) -> Result<NumberRepr, String> {
    match s {
        "fixed" => Ok(NumberRepr::Fixed),
        "float" => Ok(NumberRepr::Float),
        "sci" => Ok(NumberRepr::Sci),
        "rational" => Ok(NumberRepr::Rational),
        "financial" => Ok(NumberRepr::Financial),
        _ => Err(format!(
            "expected fixed|float|sci|rational|financial, got {s:?}"
        )),
    }
}

fn parse_currency_provider(s: &str) -> Result<CurrencyProvider, String> {
    match s {
        "mnb" => Ok(CurrencyProvider::Mnb),
        "static" => Ok(CurrencyProvider::Static),
        _ => Err(format!("expected mnb|static, got {s:?}")),
    }
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("expected true|false, got {s:?}")),
    }
}

fn parse_u8(s: &str) -> Result<u8, String> {
    s.parse::<u8>()
        .map_err(|_| format!("expected integer 0–255, got {s:?}"))
}

fn parse_f64(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("expected a number, got {s:?}"))
}

pub static REGISTRY: &[ConfigEntry] = &[
    ConfigEntry {
        key: "format.repr",
        get: |c| c.format.repr.to_string(),
        set: |c, v| {
            c.format.repr = parse_number_repr(v)?;
            Ok(())
        },
        completions: &["fixed", "float", "sci", "rational", "financial"],
    },
    ConfigEntry {
        key: "format.float.precision",
        get: |c| c.format.float.precision.to_string(),
        set: |c, v| {
            c.format.float.precision = parse_u8(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "format.float.sci_upgrade",
        get: |c| c.format.float.sci_upgrade.to_string(),
        set: |c, v| {
            c.format.float.sci_upgrade = parse_bool(v)?;
            Ok(())
        },
        completions: &["true", "false"],
    },
    ConfigEntry {
        key: "format.float.sci_upgrade_lower",
        get: |c| c.format.float.sci_upgrade_lower.to_string(),
        set: |c, v| {
            c.format.float.sci_upgrade_lower = parse_f64(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "format.float.sci_upgrade_upper",
        get: |c| c.format.float.sci_upgrade_upper.to_string(),
        set: |c, v| {
            c.format.float.sci_upgrade_upper = parse_f64(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "format.sci.precision",
        get: |c| c.format.sci.precision.to_string(),
        set: |c, v| {
            c.format.sci.precision = parse_u8(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "format.fin.precision",
        get: |c| c.format.fin.precision.to_string(),
        set: |c, v| {
            c.format.fin.precision = parse_u8(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "format.int.sci_upgrade",
        get: |c| c.format.int.sci_upgrade.to_string(),
        set: |c, v| {
            c.format.int.sci_upgrade = parse_bool(v)?;
            Ok(())
        },
        completions: &["true", "false"],
    },
    ConfigEntry {
        key: "format.int.sci_upgrade_upper",
        get: |c| c.format.int.sci_upgrade_upper.to_string(),
        set: |c, v| {
            c.format.int.sci_upgrade_upper = parse_f64(v)?;
            Ok(())
        },
        completions: &[],
    },
    ConfigEntry {
        key: "currency.provider",
        get: |c| c.currency.provider.to_string(),
        set: |c, v| {
            c.currency.provider = parse_currency_provider(v)?;
            Ok(())
        },
        completions: &["mnb", "static"],
    },
];
