//! Shared state: the config, the monitor snapshot and the colour manager.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard};

use crate::config::{AppConfig, MonitorProfile};
use crate::monitors::{self, MonitorSnapshot};
use crate::sys_colors::SystemColorManager;

pub struct AppState {
    config_path: PathBuf,
    config: Mutex<AppConfig>,
    monitors: RwLock<MonitorSnapshot>,
    pub colors: SystemColorManager,
}

impl AppState {
    /// Loads the config, repairs colours left behind by a crashed run and
    /// captures the original scheme.
    pub fn init(config_path: PathBuf) -> Arc<Self> {
        let mut config = AppConfig::load(&config_path);

        // clean_shutdown == false means the previous run left our colours in
        // place. Restore the saved originals before reading the current colours
        // as the "original" ones.
        let recovery = (!config.clean_shutdown)
            .then_some(config.saved_originals)
            .flatten();

        let colors = SystemColorManager::capture(recovery);

        config.saved_originals = Some(colors.originals());
        config.clean_shutdown = false;

        let state = Arc::new(Self {
            config_path,
            config: Mutex::new(config),
            monitors: RwLock::new(MonitorSnapshot::default()),
            colors,
        });

        state.refresh_monitors();
        state.save_config();
        state
    }

    pub fn config(&self) -> MutexGuard<'_, AppConfig> {
        self.config.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn monitors(&self) -> RwLockReadGuard<'_, MonitorSnapshot> {
        self.monitors.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Edits the config and immediately writes it to disk.
    pub fn update_config<T>(&self, edit: impl FnOnce(&mut AppConfig) -> T) -> T {
        let result = {
            let mut config = self.config();
            edit(&mut config)
        };
        self.save_config();
        result
    }

    pub fn save_config(&self) {
        let config = self.config().clone();
        if let Err(err) = config.save(&self.config_path) {
            tracing::error!(?err, path = ?self.config_path, "could not save the config");
        }
    }

    /// Rebuilds the monitor list and creates profiles for new monitors.
    ///
    /// Profiles of disconnected monitors are deliberately kept: plug the
    /// monitor back in and its colours are still there.
    pub fn refresh_monitors(&self) -> MonitorSnapshot {
        let snapshot = monitors::enumerate();
        tracing::info!(count = snapshot.monitors.len(), "display configuration");

        let mut added = false;
        {
            let mut config = self.config();
            for (index, monitor) in snapshot.monitors.iter().enumerate() {
                config
                    .monitors
                    .entry(monitor.id.clone())
                    .or_insert_with(|| {
                        added = true;
                        tracing::info!(id = %monitor.id, label = %monitor.label, "new monitor");
                        MonitorProfile::for_index(index)
                    });
            }
        }
        if added {
            self.save_config();
        }

        *self.monitors.write().unwrap_or_else(|e| e.into_inner()) = snapshot.clone();
        snapshot
    }

    /// Puts the system colours back and marks the config as cleanly closed.
    ///
    /// Called on exit. Idempotent.
    pub fn shutdown(&self) {
        self.colors.restore();
        self.update_config(|config| config.clean_shutdown = true);
    }
}
