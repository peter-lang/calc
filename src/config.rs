use std::env;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock, RwLockReadGuard};

use serde::Deserialize;

use crate::error::CalcError;
use crate::files;

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Debug, Default, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub format: FormatOptions,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct FormatOptions {
    /// Fixed-point decimal places for floats / rationals.
    pub precision: u8,
    /// Enable scientific notation for floats/rationals outside [sci_lower, sci_upper].
    pub scientific: bool,
    /// |x| below this → scientific (only when `scientific` is true).
    pub sci_lower: f64,
    /// |x| above this → scientific (only when `scientific` is true).
    pub sci_upper: f64,
    /// Mantissa decimal places in scientific mode.
    pub sci_precision: u8,
    /// Print `Rational` values as `a/b` instead of decimals.
    pub rational: bool,
    /// Opt integers into scientific notation above `int_sci_upper`.
    pub int_scientific: bool,
    /// Integer scientific-notation threshold (only when `int_scientific` is true).
    pub int_sci_upper: f64,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            precision: 4,
            scientific: true,
            sci_lower: 1e-6,
            sci_upper: 1e6,
            sci_precision: 4,
            rational: false,
            int_scientific: false,
            int_sci_upper: 1e15,
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

[format]
precision      = 4      # fixed-point decimal places for floats / rationals
scientific     = true   # use scientific notation outside [sci_lower, sci_upper]
sci_lower      = 1e-6   # |x| below this threshold → scientific
sci_upper      = 1e6    # |x| above this threshold → scientific
sci_precision  = 4      # mantissa decimal places in scientific mode
rational       = false  # show exact fractions as a/b instead of decimals
int_scientific = false  # opt integers into scientific above int_sci_upper
int_sci_upper  = 1e15   # integer scientific threshold (only with int_scientific)
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
