//! Persisting [`AppState`] to disk.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{KfError, Result};
use crate::legacy;
use crate::model::{AppState, CURRENT_STATE_VERSION};

/// Overrides the location of the state file (handy for testing and portable setups).
pub const STATE_PATH_ENV: &str = "KUBE_FORWARDER_STATE";

const APP_DIR: &str = "com.pixelpoint.kube-forwarder";

/// Storage at [`default_state_path`]. Data of the Electron application is imported on the
/// first launch, unless a custom location is set with [`STATE_PATH_ENV`] (an isolated setup).
pub fn default_storage() -> Storage {
    if std::env::var_os(STATE_PATH_ENV).is_some() {
        Storage::without_legacy_import(default_state_path())
    } else {
        Storage::new(default_state_path())
    }
}

/// `~/Library/Application Support/com.pixelpoint.kube-forwarder/state.json` on macOS.
pub fn default_state_path() -> PathBuf {
    if let Some(path) = std::env::var_os(STATE_PATH_ENV) {
        return PathBuf::from(path);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
        .join("state.json")
}

#[derive(Debug)]
pub struct LoadOutcome {
    pub state: AppState,
    /// Set when the state was imported from the Electron version of the application.
    pub imported_from: Option<PathBuf>,
    /// A problem that should be shown to the user. The application starts anyway.
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Storage {
    path: PathBuf,
    legacy_paths: Vec<PathBuf>,
}

impl Storage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy_paths: legacy::legacy_state_paths(),
        }
    }

    /// Storage that never imports data of the Electron application (used by tests).
    pub fn without_legacy_import(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy_paths: Vec::new(),
        }
    }

    pub fn with_legacy_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.legacy_paths = paths;
        self
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> LoadOutcome {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => match parse_state(&text) {
                Ok(state) => LoadOutcome {
                    state,
                    imported_from: None,
                    warning: None,
                },
                Err(error) => {
                    let backup = self.path.with_extension(format!(
                        "corrupt-{}.json",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or_default()
                    ));
                    let moved = std::fs::rename(&self.path, &backup).is_ok();
                    let location = if moved {
                        format!("It was moved to {}.", backup.display())
                    } else {
                        String::new()
                    };
                    LoadOutcome {
                        state: AppState::default(),
                        imported_from: None,
                        warning: Some(format!(
                            "Saved clusters could not be read: {error}. {location}"
                        )),
                    }
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.import_legacy(),
            Err(error) => LoadOutcome {
                state: AppState::default(),
                imported_from: None,
                warning: Some(format!("Failed to read {}: {error}", self.path.display())),
            },
        }
    }

    /// First launch of this version: import the data of the Electron application.
    fn import_legacy(&self) -> LoadOutcome {
        for candidate in &self.legacy_paths {
            let Ok(text) = std::fs::read_to_string(candidate) else {
                continue;
            };
            match legacy::parse_legacy_state(&text) {
                Ok(state) if !state.clusters.is_empty() => {
                    let warning = self
                        .save(&state)
                        .err()
                        .map(|error| format!("Failed to save imported clusters: {error}"));
                    return LoadOutcome {
                        state,
                        imported_from: Some(candidate.clone()),
                        warning,
                    };
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(path = %candidate.display(), %error, "failed to import legacy state")
                }
            }
        }
        LoadOutcome {
            state: AppState::default(),
            imported_from: None,
            warning: None,
        }
    }

    /// Writes the state atomically. The file may contain credentials, so it is only
    /// readable by the current user.
    pub fn save(&self, state: &AppState) -> Result<()> {
        let fail = |error: std::io::Error| {
            KfError::new(format!("Failed to save {}.", self.path.display()))
                .with_details(error.to_string())
        };
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(fail)?;
        }
        let json = serde_json::to_string_pretty(state).map_err(|error| {
            KfError::new("Failed to serialize the state.").with_details(error.to_string())
        })?;

        let tmp = self.path.with_extension("json.tmp");
        {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&tmp).map_err(fail)?;
            file.write_all(json.as_bytes()).map_err(fail)?;
            file.sync_all().map_err(fail)?;
        }
        std::fs::rename(&tmp, &self.path).map_err(fail)?;
        Ok(())
    }
}

fn parse_state(text: &str) -> std::result::Result<AppState, String> {
    let mut state: AppState = serde_json::from_str(text).map_err(|error| error.to_string())?;
    if state.version > CURRENT_STATE_VERSION {
        return Err(format!(
            "it was saved by a newer version of Kube Forwarder (format {})",
            state.version
        ));
    }
    state.version = CURRENT_STATE_VERSION;
    state.clusters = state
        .clusters
        .into_iter()
        .map(|mut cluster| {
            cluster.config = cluster.config.normalized();
            cluster
        })
        .collect();
    state.services = state
        .services
        .iter()
        .map(|service| service.normalized())
        .collect();
    state.cleanup();
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Cluster, ClusterConfig, Forward, Service, WorkloadType};

    fn sample_state() -> AppState {
        AppState {
            version: CURRENT_STATE_VERSION,
            clusters: vec![Cluster {
                id: "c1".into(),
                name: "local".into(),
                folded: false,
                config: ClusterConfig::from_path("/tmp/config", "ctx"),
            }],
            services: vec![Service {
                id: "s1".into(),
                cluster_id: "c1".into(),
                alias: Some("web".into()),
                namespace: "default".into(),
                workload_type: WorkloadType::Deployment,
                workload_name: "nginx".into(),
                forwards: vec![Forward::new(8080, 80)],
                local_address: None,
            }],
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::without_legacy_import(dir.path().join("nested").join("state.json"));
        let state = sample_state();
        storage.save(&state).unwrap();

        let outcome = storage.load();
        assert_eq!(outcome.state, state);
        assert!(outcome.warning.is_none());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(storage.path())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn missing_file_starts_empty() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::without_legacy_import(dir.path().join("state.json"));
        let outcome = storage.load();
        assert_eq!(outcome.state, AppState::default());
        assert!(outcome.imported_from.is_none());
    }

    #[test]
    fn corrupt_file_is_moved_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        std::fs::write(&path, "{ not json").unwrap();
        let storage = Storage::without_legacy_import(&path);
        let outcome = storage.load();
        assert_eq!(outcome.state, AppState::default());
        assert!(outcome.warning.unwrap().contains("moved to"));
        assert!(!path.exists());
    }

    #[test]
    fn imports_the_electron_store_once() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("vuex.json");
        std::fs::write(
            &legacy,
            serde_json::json!({ "state": { "version": 2,
                "Clusters": { "items": { "c1": { "id": "c1", "name": "old", "config": {
                    "storingMethod": "content", "content": "apiVersion: v1\n" } } } },
                "Services": { "items": {} } } })
            .to_string(),
        )
        .unwrap();

        let path = dir.path().join("state.json");
        let storage = Storage::without_legacy_import(&path).with_legacy_paths(vec![legacy.clone()]);
        let outcome = storage.load();
        assert_eq!(outcome.imported_from.as_deref(), Some(legacy.as_path()));
        assert_eq!(outcome.state.clusters[0].name, "old");
        assert!(path.exists(), "imported state is saved");

        let outcome = storage.load();
        assert!(outcome.imported_from.is_none(), "import happens only once");
        assert_eq!(outcome.state.clusters.len(), 1);
    }
}
