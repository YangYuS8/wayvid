/// Configuration file watcher for hot reload support
use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, info, warn};

/// Configuration file watcher that monitors for changes
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
    pub receiver: Receiver<PathBuf>,
}

impl ConfigWatcher {
    /// Start watching the configuration file
    pub fn watch(config_path: PathBuf) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = Self::create_watcher(config_path.clone(), tx)?;

        info!("Config watcher started for: {}", config_path.display());

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Create and configure the file system watcher
    fn create_watcher(config_path: PathBuf, tx: Sender<PathBuf>) -> Result<RecommendedWatcher> {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // We only care about modify events
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            debug!("Config file changed: {}", path.display());
                            if let Err(e) = tx.send(path) {
                                warn!("Failed to send config change notification: {}", e);
                            }
                        }
                    }
                }
                Err(e) => warn!("Watch error: {}", e),
            }
        })?;

        // Watch the config file (or its parent directory if it doesn't exist yet)
        let watch_path = if config_path.exists() {
            &config_path
        } else if let Some(parent) = config_path.parent() {
            parent
        } else {
            &config_path
        };

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    /// Check if a config change occurred (non-blocking)
    pub fn try_recv(&self) -> Option<PathBuf> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn test_config_watcher() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.yaml");

        // Create initial config
        fs::write(&config_path, "test: value\n").unwrap();

        // Start watcher
        let watcher = ConfigWatcher::watch(config_path.clone()).unwrap();

        // Give watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Modify config
        fs::write(&config_path, "test: new_value\n").unwrap();

        // Wait for notification
        thread::sleep(Duration::from_millis(500));

        // Check if change was detected
        let changed = watcher.try_recv();
        assert!(changed.is_some());
    }
}
