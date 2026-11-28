use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc::channel},
    time::Duration,
};

use arc_swap::{ArcSwap, Guard};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::{model::Config, validate::validate};

const DEBOUND_TIME: Duration = Duration::from_secs(2);
const DELAY_TIME: Duration = Duration::from_millis(100);

pub struct ConfigManager {
    config: ArcSwap<Config>,
    path: String,
}

impl ConfigManager {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::load_from_file(path)?;
        Ok(Self {
            config: ArcSwap::from_pointee(config),
            path: path.into(),
        })
    }

    fn load_from_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        let report = validate(&config);
        if !report.errors.is_empty() {
            let messages: Vec<&str> = report.errors.iter().map(|e| e.message.as_str()).collect();
            return Err(format!("Config validation failed: {}", messages.join("; ")).into());
        }
        Ok(config)
    }

    pub fn get(&self) -> Guard<Arc<Config>> {
        self.config.load()
    }

    pub fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        let new_config = Self::load_from_file(&self.path)?;
        self.config.store(Arc::new(new_config));
        Ok(())
    }

    pub fn watch_for_changes(
        manager: Arc<ConfigManager>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            NotifyConfig::default().with_poll_interval(DEBOUND_TIME),
        )?;

        let path = manager.path.clone();
        watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

        std::thread::spawn(move || {
            let _watcher = watcher;

            for event in rx {
                if event.kind.is_modify() || event.kind.is_create() {
                    println!("Config file changed, reloading ...");

                    std::thread::sleep(DELAY_TIME);

                    match manager.reload() {
                        Ok(()) => println!("Configuration reloaded successful!"),
                        Err(e) => eprintln!("Failed to reload config: {}", e),
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempConfigFile {
        path: std::path::PathBuf,
    }

    impl TempConfigFile {
        fn new(contents: &str) -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "pressflow_watcher_test_{}_{}.toml",
                std::process::id(),
                id
            ));
            fs::write(&path, contents).expect("failed to write temp config");
            Self { path }
        }

        fn write(&self, contents: &str) {
            fs::write(&self.path, contents).expect("failed to update temp config");
        }

        fn path_str(&self) -> &str {
            self.path.to_str().expect("temp path should be valid UTF-8")
        }
    }

    impl Drop for TempConfigFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_reload_updates_values() {
        let file = TempConfigFile::new(
            r#"
[appearance]
columns = 4
window_opacity = 0.85

[behavior]
cheatsheet_delay_ms = 1000
"#,
        );

        let manager = ConfigManager::new(file.path_str()).expect("initial config should load");
        assert_eq!(manager.get().appearance.columns, 4);
        assert!((manager.get().appearance.window_opacity - 0.85).abs() < f32::EPSILON);
        assert_eq!(manager.get().behavior.cheatsheet_delay_ms, 1000);

        file.write(
            r#"
[appearance]
columns = 2
window_opacity = 0.5

[behavior]
cheatsheet_delay_ms = 250
"#,
        );

        manager
            .reload()
            .expect("valid updated config should reload");
        assert_eq!(manager.get().appearance.columns, 2);
        assert!((manager.get().appearance.window_opacity - 0.5).abs() < f32::EPSILON);
        assert_eq!(manager.get().behavior.cheatsheet_delay_ms, 250);
    }

    #[test]
    fn test_invalid_config_rejected() {
        let file = TempConfigFile::new(
            r#"
[appearance]
columns = 3
window_opacity = 0.9
"#,
        );

        let manager = ConfigManager::new(file.path_str()).expect("initial config should load");
        assert_eq!(manager.get().appearance.columns, 3);

        file.write("not valid toml = [");
        assert!(manager.reload().is_err());
        assert_eq!(manager.get().appearance.columns, 3);
        assert!((manager.get().appearance.window_opacity - 0.9).abs() < f32::EPSILON);

        file.write(
            r#"
[appearance]
columns = 0
window_opacity = 2.5
"#,
        );
        assert!(manager.reload().is_err());
        assert_eq!(manager.get().appearance.columns, 3);
        assert!((manager.get().appearance.window_opacity - 0.9).abs() < f32::EPSILON);
    }
}
