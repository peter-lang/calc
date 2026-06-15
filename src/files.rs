use std::fs;
use std::path::{Path, PathBuf};

use etcetera::{choose_base_strategy, BaseStrategy};

use crate::error::CalcError;

/// Returns the path to the user config file without creating any directories.
pub fn config() -> Result<PathBuf, CalcError> {
    let strategy = choose_base_strategy().map_err(|_| CalcError::HomeDirNotFound)?;
    Ok(strategy.config_dir().join("calc").join("conf.toml"))
}

pub fn cache(name: impl AsRef<Path>) -> Result<PathBuf, CalcError> {
    let strategy = choose_base_strategy().map_err(|_| CalcError::HomeDirNotFound)?;
    let dir = strategy.cache_dir().join("calc");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir.join(name))
}
