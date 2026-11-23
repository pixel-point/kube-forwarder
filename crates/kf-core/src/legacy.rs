//! Reading data written by the Electron version of Kube Forwarder.
//!
//! The Electron application persisted its Vuex store with `vuex-electron` into
//! `<userData>/vuex.json` as `{ "state": { "version", "Clusters": { "items": {..} },
//! "Services": { "items": {..} } } }`. Version 1 stored the kube config of a cluster as a
//! plain YAML string, version 2 as an object (see [`ClusterConfig`]).

use std::path::PathBuf;

use serde_json::Value;

use crate::error::{KfError, Result};
use crate::kubeconfig;
use crate::model::{AppState, Cluster, ClusterConfig, Service, new_id};

/// Where the Electron application kept its store (`app.getPath('userData')`).
pub fn legacy_state_paths() -> Vec<PathBuf> {
    let Some(base) = dirs::config_dir() else {
        return Vec::new();
    };
    ["Kube Forwarder", "kube-forwarder"]
        .iter()
        .map(|dir| base.join(dir).join("vuex.json"))
        .collect()
}

/// Version 1 → 2 migration: the config was stored as YAML text.
pub fn migrate_v1_config(content: &str) -> ClusterConfig {
    let current_context = kubeconfig::parse_str(content)
        .ok()
        .and_then(|parsed| parsed.current_context);
    ClusterConfig::from_content(content, current_context).normalized()
}

/// Parses `vuex.json` of the Electron application.
pub fn parse_legacy_state(text: &str) -> Result<AppState> {
    let root: Value = serde_json::from_str(text).map_err(|error| {
        KfError::new("The saved data of the previous version is not valid JSON.")
            .with_details(error.to_string())
    })?;
    let state = root.get("state").unwrap_or(&root);
    Ok(parse_vuex_state(state))
}

/// Converts the Vuex state (`{ version, Clusters, Services }`) into [`AppState`].
/// Invalid items are skipped.
pub fn parse_vuex_state(state: &Value) -> AppState {
    let mut result = AppState::default();

    if let Some(items) = state.pointer("/Clusters/items").and_then(Value::as_object) {
        for (id, value) in items {
            match parse_cluster(id, value) {
                Some(cluster) => result.clusters.push(cluster),
                None => tracing::warn!(id, "skipping an invalid cluster of the previous version"),
            }
        }
    }

    if let Some(items) = state.pointer("/Services/items").and_then(Value::as_object) {
        for (id, value) in items {
            let mut value = value.clone();
            if let Some(object) = value.as_object_mut() {
                object
                    .entry("id")
                    .or_insert_with(|| Value::String(id.clone()));
            }
            match serde_json::from_value::<Service>(value) {
                Ok(service) => result.services.push(service.normalized()),
                Err(error) => {
                    tracing::warn!(id, %error, "skipping an invalid resource of the previous version")
                }
            }
        }
    }

    result.cleanup();
    result
}

fn parse_cluster(id: &str, value: &Value) -> Option<Cluster> {
    let name = value.get("name")?.as_str()?.to_string();
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if id.is_empty() {
                new_id()
            } else {
                id.to_string()
            }
        });
    let folded = value
        .get("folded")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let config = match value.get("config") {
        Some(Value::String(content)) => migrate_v1_config(content),
        None | Some(Value::Null) => ClusterConfig::default(),
        Some(config) => serde_json::from_value::<ClusterConfig>(config.clone())
            .ok()?
            .normalized(),
    };
    Some(Cluster {
        id,
        name,
        folded,
        config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{StoringMethod, WorkloadType};

    const V1_CONFIG: &str = "apiVersion: v1\ncurrent-context: prod\ncontexts:\n- name: prod\n  context:\n    cluster: prod\n";

    #[test]
    fn migrates_version_1_configs() {
        let config = migrate_v1_config(V1_CONFIG);
        assert_eq!(config.storing_method, StoringMethod::Content);
        assert_eq!(config.current_context.as_deref(), Some("prod"));
        assert_eq!(config.content.as_deref(), Some(V1_CONFIG));
    }

    #[test]
    fn parses_vuex_electron_file() {
        let text = serde_json::json!({
            "state": {
                "version": 2,
                "Clusters": { "items": {
                    "c1": { "id": "c1", "name": "v1 cluster", "config": V1_CONFIG },
                    "c2": { "id": "c2", "name": "path cluster", "folded": true, "config": {
                        "storingMethod": "path", "path": "/Users/me/.kube/config",
                        "currentContext": "dev", "content": ""
                    }},
                    "broken": { "config": {} }
                }},
                "Services": { "items": {
                    "s1": { "id": "s1", "clusterId": "c1", "namespace": "db", "workloadType": "pod",
                            "workloadName": "postgres-0", "alias": "",
                            "forwards": [{ "id": "f1", "localPort": 5432, "remotePort": 5432 }],
                            "localAddress": null },
                    "s2": { "id": "s2", "clusterId": "c2", "namespace": "web", "workloadType": "service",
                            "workloadName": "frontend", "localAddress": "127.0.1.1",
                            "forwards": [{ "id": "f2", "localPort": 8080, "remotePort": 80 }] },
                    "orphan": { "id": "orphan", "clusterId": "gone", "namespace": "x", "workloadType": "pod",
                            "workloadName": "x", "forwards": [{ "id": "f3", "localPort": 1, "remotePort": 1 }] }
                }}
            }
        })
        .to_string();

        let state = parse_legacy_state(&text).unwrap();
        assert_eq!(state.clusters.len(), 2, "the invalid cluster is skipped");
        assert_eq!(state.clusters[0].name, "v1 cluster");
        assert_eq!(
            state.clusters[0].config.current_context.as_deref(),
            Some("prod")
        );
        assert!(state.clusters[1].folded);
        assert_eq!(state.clusters[1].config.storing_method, StoringMethod::Path);
        assert_eq!(state.clusters[1].config.content, None);

        assert_eq!(state.services.len(), 2, "the orphan resource is removed");
        assert_eq!(state.services[0].alias, None);
        assert_eq!(state.services[1].workload_type, WorkloadType::Service);
        assert_eq!(
            state.services[1].local_address.as_deref(),
            Some("127.0.1.1")
        );
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse_legacy_state("{").is_err());
    }
}
