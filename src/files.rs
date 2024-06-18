use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::ProjectDirs;

pub fn cache(name: impl AsRef<Path>) -> PathBuf {
    static DIRS: OnceLock<ProjectDirs> = OnceLock::new();
    DIRS.get_or_init(|| {
        let path = ProjectDirs::from_path(PathBuf::from("calc")).unwrap();
        if !path.cache_dir().exists() {
            fs::create_dir_all(path.cache_dir()).unwrap();
        }
        path
    })
    .cache_dir()
    .join(name)
}
