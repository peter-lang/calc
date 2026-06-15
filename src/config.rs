use std::env;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock, RwLockReadGuard};

use serde::Deserialize;

use crate::error::CalcError;
use crate::files;

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// Top-level config struct. Extended by later plans via sub-tables.
/// Every field must have `#[serde(default)]` so partial/empty files are valid.
#[derive(Debug, Default, Deserialize)]
pub struct Config {}

pub fn init() -> Result<(), CalcError> {
    let config = load()?;
    CONFIG.get_or_init(|| RwLock::new(config));
    Ok(())
}

#[allow(dead_code)]
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
#
# Missing keys fall back to built-in defaults.
# Edit this file to customise calc's behaviour.
";
