use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock, RwLockReadGuard};

use serde::{Deserialize, Serialize};

use crate::error::CalcError;
use crate::files;

mod registry;

pub use registry::REGISTRY;

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub format: FormatOptions,
    #[serde(default)]
    pub currency: CurrencyConfig,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CurrencyProvider {
    #[default]
    Mnb,
    Static,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct CurrencyConfig {
    pub provider: CurrencyProvider,
    #[serde(rename = "static")]
    pub static_rates: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NumberRepr {
    Fixed,
    #[default]
    Float,
    Sci,
    Rational,
    Financial,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct FormatOptions {
    pub repr: NumberRepr,
    pub float: FloatConfig,
    pub sci: SciConfig,
    pub fin: FinConfig,
    pub int: IntConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct FloatConfig {
    pub precision: u8,
    pub sci_upgrade: bool,
    pub sci_upgrade_lower: f64,
    pub sci_upgrade_upper: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct SciConfig {
    pub precision: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct FinConfig {
    pub precision: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct IntConfig {
    pub sci_upgrade: bool,
    pub sci_upgrade_upper: f64,
}

impl std::fmt::Display for NumberRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Fixed => "fixed",
            Self::Float => "float",
            Self::Sci => "sci",
            Self::Rational => "rational",
            Self::Financial => "financial",
        })
    }
}

impl std::fmt::Display for CurrencyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Mnb => "mnb",
            Self::Static => "static",
        })
    }
}

pub fn set_key(key: &str, value: &str) -> Result<String, String> {
    let entry = REGISTRY.iter().find(|e| e.key == key).ok_or_else(|| {
        let keys: Vec<_> = REGISTRY.iter().map(|e| e.key).collect();
        format!("unknown key {key:?}; valid keys: {}", keys.join(", "))
    })?;
    let mut cfg = CONFIG
        .get()
        .expect("config::init() must be called before config::set_key()")
        .write()
        .expect("config RwLock poisoned");
    (entry.set)(&mut cfg, value).map_err(|e| format!("{key}: {e}"))?;
    Ok((entry.get)(&cfg))
}

pub fn persist() -> Result<(), CalcError> {
    let path = config_path()?;
    let text = {
        let cfg = current();
        toml::to_string(&*cfg).map_err(|e| CalcError::ConfigError(e.to_string()))?
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}

pub enum FormatSpec {
    Fixed { precision: Option<u8> },
    Float,
    Sci { precision: Option<u8> },
    Rational,
    Financial { precision: Option<u8> },
}

pub fn apply_spec(base: &FormatOptions, spec: &FormatSpec) -> FormatOptions {
    let mut opts = base.clone();
    match spec {
        FormatSpec::Fixed { precision } => {
            opts.repr = NumberRepr::Fixed;
            if let Some(p) = precision {
                opts.float.precision = *p;
            }
        }
        FormatSpec::Float => {
            opts.repr = NumberRepr::Float;
        }
        FormatSpec::Sci { precision } => {
            opts.repr = NumberRepr::Sci;
            if let Some(p) = precision {
                opts.sci.precision = *p;
            }
        }
        FormatSpec::Rational => {
            opts.repr = NumberRepr::Rational;
        }
        FormatSpec::Financial { precision } => {
            opts.repr = NumberRepr::Financial;
            if let Some(p) = precision {
                opts.fin.precision = *p;
            }
        }
    }
    opts
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            repr: NumberRepr::Float,
            float: FloatConfig::default(),
            sci: SciConfig::default(),
            fin: FinConfig::default(),
            int: IntConfig::default(),
        }
    }
}

impl Default for FloatConfig {
    fn default() -> Self {
        Self {
            precision: 4,
            sci_upgrade: true,
            sci_upgrade_lower: 1e-6,
            sci_upgrade_upper: 1e6,
        }
    }
}

impl Default for SciConfig {
    fn default() -> Self {
        Self { precision: 4 }
    }
}

impl Default for FinConfig {
    fn default() -> Self {
        Self { precision: 2 }
    }
}

impl Default for IntConfig {
    fn default() -> Self {
        Self {
            sci_upgrade: false,
            sci_upgrade_upper: 1e15,
        }
    }
}

pub fn init() -> Result<(), CalcError> {
    let config = load()?;
    CONFIG.get_or_init(|| RwLock::new(config));
    Ok(())
}

pub fn current() -> RwLockReadGuard<'static, Config> {
    CONFIG
        .get()
        .expect("config::init() must be called before config::current()")
        .read()
        .expect("config RwLock poisoned")
}

fn config_path() -> Result<PathBuf, CalcError> {
    match env::var_os("CALC_CONFIG") {
        Some(p) => Ok(PathBuf::from(p)),
        None => files::config(),
    }
}

fn load() -> Result<Config, CalcError> {
    match env::var_os("CALC_CONFIG") {
        Some(p) => load_from_path(Path::new(&p)),
        None => {
            let path = files::config()?;
            if !path.exists() {
                bootstrap(&path)?;
            }
            load_from_path(&path)
        }
    }
}

fn load_from_path(path: &Path) -> Result<Config, CalcError> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Config::default());
    }
    toml::from_str(&text).map_err(|e| CalcError::ConfigError(e.to_string()))
}

fn bootstrap(path: &Path) -> Result<(), CalcError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_TEMPLATE)?;
    Ok(())
}

const DEFAULT_TEMPLATE: &str = "\
# calc configuration — ~/.config/calc/conf.toml
# Missing keys fall back to built-in defaults.

# [currency]
# provider = \"mnb\"    # mnb (default) or static
#
# # static: fixed rates for offline use / deterministic tests; direct lookup only
# # [currency.static]
# # \"EUR/USD\" = 1.08

[format]
repr = \"float\"  # fixed | float | sci | rational | financial

[format.float]
precision         = 4      # decimal places for fixed/float display
sci_upgrade       = true   # auto-upgrade to sci outside [lower, upper]
sci_upgrade_lower = 1e-6   # |x| below this → sci
sci_upgrade_upper = 1e6    # |x| above this → sci

[format.sci]
precision = 4              # mantissa decimal places

[format.fin]
precision = 2              # decimal places for financial display

[format.int]
sci_upgrade       = false  # auto-upgrade integers to sci above upper
sci_upgrade_upper = 1e15   # integer sci threshold
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_parses_to_default_format_options() {
        let config: Config = toml::from_str(DEFAULT_TEMPLATE).expect("template must be valid TOML");
        assert_eq!(config, Config::default());
    }
}
