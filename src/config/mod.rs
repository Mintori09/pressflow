use std::path::PathBuf;

pub mod key_binding;
pub mod model;
pub mod validate;
pub mod watcher;

pub fn default_config_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("examples/config.example.toml")
    } else {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("pressflow/config.toml");
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config/pressflow/config.toml")
    }
}
