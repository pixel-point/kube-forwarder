//! Export and import of clusters with their resources.
//!
//! The format is the one of the Electron application (version 2):
//!
//! ```json
//! { "version": 2, "clusters": [ { "name": "...", "config": { ... }, "_services": [ ... ] } ] }
//! ```
//!
//! Version 1 exports (`{ "_clusters": [ { "name", "config": "<yaml>", "_services" } ] }`)
//! can be imported as well.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{KfError, Result};
use crate::kubeconfig;
use crate::legacy::migrate_v1_config;
use crate::model::{
    AppState, CURRENT_STATE_VERSION, ClusterConfig, Forward, Service, StoringMethod, WorkloadType,
    new_id,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportFile {
    pub version: u32,
    pub clusters: Vec<ExportedCluster>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedCluster {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<ClusterConfig>,
    #[serde(rename = "_services", default)]
    pub services: Vec<ExportedService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedService {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub namespace: String,
    pub workload_type: WorkloadType,
    pub workload_name: String,
    pub forwards: Vec<Forward>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_address: Option<String>,
}

impl ExportedService {
    pub fn from_service(service: &Service) -> Self {
        let service = service.normalized();
        Self {
            alias: service.alias,
            namespace: service.namespace,
            workload_type: service.workload_type,
            workload_name: service.workload_name,
            forwards: service.forwards,
            local_address: service.local_address,
        }
    }

    /// Creates a new resource of a cluster from the exported data.
    pub fn to_service(&self, cluster_id: &str) -> Service {
        Service {
            id: new_id(),
            cluster_id: cluster_id.to_string(),
            alias: self.alias.clone(),
            namespace: self.namespace.clone(),
            workload_type: self.workload_type,
            workload_name: self.workload_name.clone(),
            forwards: self
                .forwards
                .iter()
                .map(|forward| Forward::new(forward.local_port, forward.remote_port))
                .collect(),
            local_address: self.local_address.clone(),
        }
        .normalized()
    }
}

/// Builds an export of one cluster. With `include_config` the kube config is embedded
/// (a config referenced by path is read and embedded as text).
pub fn export_cluster(
    state: &AppState,
    cluster_id: &str,
    include_config: bool,
) -> Result<ExportFile> {
    let cluster = state
        .cluster(cluster_id)
        .ok_or_else(|| KfError::new("The cluster doesn't exist."))?;

    let config = if include_config {
        let config = cluster.config.normalized();
        Some(match config.storing_method {
            StoringMethod::Path => {
                let path = config
                    .path
                    .as_deref()
                    .ok_or_else(|| KfError::new("The path to the kube config is empty."))?;
                let content = kubeconfig::read_text_file(&kubeconfig::expand_tilde(path))?;
                ClusterConfig::from_content(content, config.current_context.clone())
            }
            StoringMethod::Content => config,
        })
    } else {
        None
    };

    Ok(ExportFile {
        version: CURRENT_STATE_VERSION,
        clusters: vec![ExportedCluster {
            name: cluster.name.clone(),
            config,
            services: state
                .services_of(cluster_id)
                .map(ExportedService::from_service)
                .collect(),
        }],
    })
}

pub fn to_json(file: &ExportFile) -> String {
    serde_json::to_string_pretty(file).expect("export file is always serializable")
}

/// `cluster-<name>.kpf-export.v2.json` (as in the original application).
pub fn default_file_name(cluster_name: &str) -> String {
    let safe: String = cluster_name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '-'
            } else {
                c
            }
        })
        .collect();
    format!("cluster-{safe}.kpf-export.v{CURRENT_STATE_VERSION}.json")
}

/// A cluster read from an export file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedCluster {
    pub name: String,
    pub config: Option<ClusterConfig>,
    pub services: Vec<ExportedService>,
}

pub fn parse_import(text: &str) -> Result<ImportedCluster> {
    let value: Value = serde_json::from_str(text).map_err(|error| {
        KfError::new("The file is not a valid JSON file.").with_details(error.to_string())
    })?;

    if value.get("version").is_none() && value.get("_clusters").is_some() {
        return parse_version_1(&value);
    }

    let version = value.get("version").and_then(Value::as_u64).unwrap_or(1);
    if version != u64::from(CURRENT_STATE_VERSION) {
        return Err(KfError::new(format!(
            "Sorry, an export of version {version} isn't supported. Actual version is {CURRENT_STATE_VERSION}."
        )));
    }

    let file: ExportFile = serde_json::from_value(value).map_err(|error| {
        KfError::new("Data for import is invalid.").with_details(error.to_string())
    })?;
    let [cluster] = <[ExportedCluster; 1]>::try_from(file.clusters).map_err(|_| {
        KfError::new("Data for import is invalid.")
            .with_details("The file must contain exactly one cluster.")
    })?;

    finish_import(
        cluster.name,
        cluster.config.map(|c| c.normalized()),
        cluster.services,
    )
}

fn parse_version_1(value: &Value) -> Result<ImportedCluster> {
    let invalid =
        |details: String| KfError::new("Data for import is invalid.").with_details(details);
    let clusters = value
        .get("_clusters")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("`_clusters` must be a list.".into()))?;
    let [cluster] = clusters.as_slice() else {
        return Err(invalid("The file must contain exactly one cluster.".into()));
    };
    let name = cluster
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let config = cluster
        .get("config")
        .and_then(Value::as_str)
        .map(migrate_v1_config)
        .filter(|config| config.content.is_some());
    let services: Vec<ExportedService> = match cluster.get("_services") {
        Some(services) => {
            serde_json::from_value(services.clone()).map_err(|error| invalid(error.to_string()))?
        }
        None => Vec::new(),
    };
    finish_import(name, config, services)
}

fn finish_import(
    name: String,
    config: Option<ClusterConfig>,
    services: Vec<ExportedService>,
) -> Result<ImportedCluster> {
    let invalid =
        |details: String| KfError::new("Data for import is invalid.").with_details(details);
    if name.trim().is_empty() {
        return Err(invalid("The cluster name is empty.".into()));
    }
    for service in &services {
        if service.namespace.trim().is_empty() || service.workload_name.trim().is_empty() {
            return Err(invalid(format!(
                "Resource \"{}\" must have a namespace and a name.",
                service.workload_name
            )));
        }
        if service.forwards.is_empty() {
            return Err(invalid(format!(
                "Resource \"{}\" doesn't forward any port.",
                service.workload_name
            )));
        }
        if service
            .forwards
            .iter()
            .any(|forward| forward.local_port == 0 || forward.remote_port == 0)
        {
            return Err(invalid(format!(
                "Resource \"{}\" has an invalid port.",
                service.workload_name
            )));
        }
    }
    Ok(ImportedCluster {
        name: name.trim().to_string(),
        config,
        services,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Cluster;

    fn state_with_config(config: ClusterConfig) -> AppState {
        let mut state = AppState::default();
        state.clusters.push(Cluster {
            id: "c1".into(),
            name: "Pixel Point".into(),
            folded: false,
            config,
        });
        state.services.push(Service {
            id: "s1".into(),
            cluster_id: "c1".into(),
            alias: None,
            namespace: "monitoring".into(),
            workload_type: WorkloadType::Deployment,
            workload_name: "grafana".into(),
            forwards: vec![Forward::new(3005, 3000)],
            local_address: Some("127.0.1.1".into()),
        });
        state
    }

    #[test]
    fn export_without_config() {
        let state = state_with_config(ClusterConfig::from_content("apiVersion: v1\n", None));
        let file = export_cluster(&state, "c1", false).unwrap();
        let json: Value = serde_json::from_str(&to_json(&file)).unwrap();
        assert_eq!(json["version"], 2);
        assert_eq!(json["clusters"][0]["name"], "Pixel Point");
        assert!(json["clusters"][0].get("config").is_none());
        let service = &json["clusters"][0]["_services"][0];
        assert_eq!(service["workloadName"], "grafana");
        assert_eq!(service["localAddress"], "127.0.1.1");
        assert!(service.get("id").is_none());
        assert!(service.get("clusterId").is_none());
    }

    #[test]
    fn export_with_path_config_embeds_file_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config");
        std::fs::write(&path, "apiVersion: v1\nkind: Config\n").unwrap();
        let state = state_with_config(ClusterConfig::from_path(path.to_str().unwrap(), "dev"));

        let file = export_cluster(&state, "c1", true).unwrap();
        let config = file.clusters[0].config.clone().unwrap();
        assert_eq!(config.storing_method, StoringMethod::Content);
        assert_eq!(
            config.content.as_deref(),
            Some("apiVersion: v1\nkind: Config\n")
        );
        assert_eq!(config.current_context.as_deref(), Some("dev"));
        assert_eq!(config.path, None);
    }

    #[test]
    fn export_import_roundtrip() {
        let state = state_with_config(ClusterConfig::from_content(
            "apiVersion: v1\n",
            Some("ctx".into()),
        ));
        let json = to_json(&export_cluster(&state, "c1", true).unwrap());
        let imported = parse_import(&json).unwrap();
        assert_eq!(imported.name, "Pixel Point");
        assert_eq!(
            imported.config.unwrap().current_context.as_deref(),
            Some("ctx")
        );
        let service = imported.services[0].to_service("new-cluster");
        assert_eq!(service.cluster_id, "new-cluster");
        assert_eq!(service.forwards[0].local_port, 3005);
        assert_ne!(service.id, "s1");
    }

    #[test]
    fn imports_files_of_the_electron_app() {
        // Written by the Electron app (compact JSON, `localAddress: null`, no config).
        let text = r#"{"version":2,"clusters":[{"name":"prod","_services":[{"namespace":"db",
            "workloadType":"pod","workloadName":"postgres-0","forwards":[{"localPort":5432,
            "remotePort":5432,"id":"7533169c"}],"localAddress":null}]}]}"#;
        let imported = parse_import(text).unwrap();
        assert_eq!(imported.config, None);
        assert_eq!(imported.services[0].local_address, None);
    }

    #[test]
    fn imports_version_1_files() {
        let text = include_str!("../tests/fixtures/v1-export.json");
        let imported = parse_import(text).unwrap();
        assert_eq!(imported.name, "Pixel Point");
        assert_eq!(imported.services.len(), 6);
        assert_eq!(imported.services[0].workload_name, "longhorn-ui");
        assert_eq!(
            imported.config.unwrap().storing_method,
            StoringMethod::Content
        );
    }

    #[test]
    fn rejects_invalid_imports() {
        assert!(
            parse_import("nope")
                .unwrap_err()
                .message
                .contains("not a valid JSON")
        );
        assert!(
            parse_import(r#"{"version":3,"clusters":[]}"#)
                .unwrap_err()
                .message
                .contains("version 3 isn't supported")
        );
        let two =
            r#"{"version":2,"clusters":[{"name":"a","_services":[]},{"name":"b","_services":[]}]}"#;
        assert_eq!(
            parse_import(two).unwrap_err().details.as_deref(),
            Some("The file must contain exactly one cluster.")
        );
        let bad_port = r#"{"version":2,"clusters":[{"name":"a","_services":[{"namespace":"n",
            "workloadType":"pod","workloadName":"p","forwards":[{"localPort":70000,"remotePort":1}]}]}]}"#;
        assert!(parse_import(bad_port).is_err());
        let bad_kind = r#"{"version":2,"clusters":[{"name":"a","_services":[{"namespace":"n",
            "workloadType":"job","workloadName":"p","forwards":[{"localPort":1,"remotePort":1}]}]}]}"#;
        assert!(parse_import(bad_kind).is_err());
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(
            default_file_name("Pixel Point"),
            "cluster-Pixel Point.kpf-export.v2.json"
        );
        assert_eq!(
            default_file_name("arn:aws:eks:x/dev"),
            "cluster-arn-aws-eks-x-dev.kpf-export.v2.json"
        );
    }
}
