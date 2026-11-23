//! Persisted data model.
//!
//! Field names and value spellings follow the JSON produced by the original
//! Electron application (`storingMethod`, `workloadType`, ...) so that exports and the
//! old persisted state can be read without conversion tables.

use serde::{Deserialize, Serialize};

/// Version of the persisted state and of the export format.
pub const CURRENT_STATE_VERSION: u32 = 2;

/// Address used when a resource doesn't configure a custom local address.
pub const DEFAULT_BIND_ADDRESS: &str = "localhost";

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Trims a string and turns empty strings into `None`.
pub fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Removes leading blank lines and trailing whitespace of a YAML document without
/// touching the indentation of the first non blank line (YAML is indentation sensitive).
pub fn tidy_text_block(content: &str) -> Option<String> {
    let mut text = content.trim_end();
    while let Some(pos) = text.find('\n') {
        if text[..pos].trim().is_empty() {
            text = &text[pos + 1..];
        } else {
            break;
        }
    }
    if text.trim().is_empty() {
        None
    } else {
        Some(format!("{text}\n"))
    }
}

/// How the kube config of a cluster is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StoringMethod {
    /// Only the path to a kube config file is stored. The file is read every time a
    /// connection is made, so tools that refresh credentials in the file keep working.
    Path,
    /// The whole kube config is stored as text.
    #[default]
    Content,
}

impl StoringMethod {
    pub const ALL: [StoringMethod; 2] = [StoringMethod::Path, StoringMethod::Content];

    pub fn label(self) -> &'static str {
        match self {
            StoringMethod::Path => "Set a path",
            StoringMethod::Content => "Paste as a text",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterConfig {
    pub storing_method: StoringMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_context: Option<String>,
}

impl ClusterConfig {
    pub fn from_path(path: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            storing_method: StoringMethod::Path,
            path: Some(path.into()),
            content: None,
            current_context: Some(context.into()),
        }
    }

    pub fn from_content(content: impl Into<String>, context: Option<String>) -> Self {
        Self {
            storing_method: StoringMethod::Content,
            path: None,
            content: Some(content.into()),
            current_context: context,
        }
    }

    /// Drops values that are irrelevant for the storing method and normalizes empty strings.
    pub fn normalized(&self) -> Self {
        let current_context = non_empty(self.current_context.as_deref());
        match self.storing_method {
            StoringMethod::Path => Self {
                storing_method: StoringMethod::Path,
                path: non_empty(self.path.as_deref()),
                content: None,
                current_context,
            },
            StoringMethod::Content => Self {
                storing_method: StoringMethod::Content,
                path: None,
                content: self.content.as_deref().and_then(tidy_text_block),
                current_context,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub folded: bool,
    pub config: ClusterConfig,
}

/// Kind of the Kubernetes resource a forward points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadType {
    Pod,
    Deployment,
    Service,
}

impl WorkloadType {
    pub const ALL: [WorkloadType; 3] = [
        WorkloadType::Pod,
        WorkloadType::Deployment,
        WorkloadType::Service,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WorkloadType::Pod => "Pod",
            WorkloadType::Deployment => "Deployment",
            WorkloadType::Service => "Service",
        }
    }
}

fn default_forward_id() -> String {
    new_id()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Forward {
    #[serde(default = "default_forward_id")]
    pub id: String,
    pub local_port: u16,
    pub remote_port: u16,
}

impl Forward {
    pub fn new(local_port: u16, remote_port: u16) -> Self {
        Self {
            id: new_id(),
            local_port,
            remote_port,
        }
    }
}

/// A bookmarked Kubernetes resource (named "service" for compatibility with the
/// original data format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub cluster_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub namespace: String,
    pub workload_type: WorkloadType,
    pub workload_name: String,
    pub forwards: Vec<Forward>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_address: Option<String>,
}

impl Service {
    /// Name displayed in the list: the alias when present, the resource name otherwise.
    pub fn label(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.workload_name)
    }

    /// Local address the forwarded ports listen on.
    pub fn bind_address(&self) -> &str {
        self.local_address
            .as_deref()
            .unwrap_or(DEFAULT_BIND_ADDRESS)
    }

    pub fn normalized(&self) -> Self {
        Self {
            id: self.id.clone(),
            cluster_id: self.cluster_id.clone(),
            alias: non_empty(self.alias.as_deref()),
            namespace: self.namespace.trim().to_string(),
            workload_type: self.workload_type,
            workload_name: self.workload_name.trim().to_string(),
            forwards: self.forwards.clone(),
            local_address: non_empty(self.local_address.as_deref()),
        }
    }

    /// Case insensitive search over the alias and the resource name.
    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }
        self.workload_name.to_lowercase().contains(&query)
            || self
                .alias
                .as_deref()
                .is_some_and(|alias| alias.to_lowercase().contains(&query))
    }
}

/// Everything that is persisted between launches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppState {
    pub version: u32,
    #[serde(default)]
    pub clusters: Vec<Cluster>,
    #[serde(default)]
    pub services: Vec<Service>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: CURRENT_STATE_VERSION,
            clusters: Vec::new(),
            services: Vec::new(),
        }
    }
}

impl AppState {
    pub fn cluster(&self, id: &str) -> Option<&Cluster> {
        self.clusters.iter().find(|cluster| cluster.id == id)
    }

    pub fn cluster_mut(&mut self, id: &str) -> Option<&mut Cluster> {
        self.clusters.iter_mut().find(|cluster| cluster.id == id)
    }

    pub fn service(&self, id: &str) -> Option<&Service> {
        self.services.iter().find(|service| service.id == id)
    }

    pub fn services_of<'a>(
        &'a self,
        cluster_id: &'a str,
    ) -> impl Iterator<Item = &'a Service> + 'a {
        self.services
            .iter()
            .filter(move |service| service.cluster_id == cluster_id)
    }

    /// Clusters sorted by name like the original list: case insensitive first, then
    /// case sensitive to keep the order stable.
    pub fn sorted_clusters(&self) -> Vec<&Cluster> {
        let mut clusters: Vec<&Cluster> = self.clusters.iter().collect();
        clusters.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.name.cmp(&b.name))
        });
        clusters
    }

    pub fn upsert_cluster(&mut self, cluster: Cluster) {
        match self.cluster_mut(&cluster.id) {
            Some(existing) => *existing = cluster,
            None => self.clusters.push(cluster),
        }
    }

    pub fn upsert_service(&mut self, service: Service) {
        match self.services.iter_mut().find(|item| item.id == service.id) {
            Some(existing) => *existing = service,
            None => self.services.push(service),
        }
    }

    /// Deletes a cluster together with its resources.
    pub fn delete_cluster(&mut self, id: &str) {
        self.clusters.retain(|cluster| cluster.id != id);
        self.services.retain(|service| service.cluster_id != id);
    }

    pub fn delete_service(&mut self, id: &str) {
        self.services.retain(|service| service.id != id);
    }

    /// Removes resources that point to a missing cluster (the `cleanup` step of the
    /// original store).
    pub fn cleanup(&mut self) {
        let cluster_ids: Vec<String> = self.clusters.iter().map(|c| c.id.clone()).collect();
        self.services
            .retain(|service| cluster_ids.contains(&service.cluster_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(alias: Option<&str>, name: &str) -> Service {
        Service {
            id: new_id(),
            cluster_id: "c".into(),
            alias: alias.map(str::to_string),
            namespace: "default".into(),
            workload_type: WorkloadType::Pod,
            workload_name: name.into(),
            forwards: vec![Forward::new(8080, 80)],
            local_address: None,
        }
    }

    #[test]
    fn serializes_with_original_field_names() {
        let svc = service(Some("db"), "postgres");
        let json = serde_json::to_value(&svc).unwrap();
        assert_eq!(json["workloadType"], "pod");
        assert_eq!(json["workloadName"], "postgres");
        assert_eq!(json["clusterId"], "c");
        assert_eq!(json["forwards"][0]["localPort"], 8080);
        assert!(json.get("localAddress").is_none());

        let config = ClusterConfig::from_path("/tmp/config", "ctx");
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["storingMethod"], "path");
        assert_eq!(json["currentContext"], "ctx");
    }

    #[test]
    fn accepts_nulls_written_by_the_original_app() {
        let svc: Service = serde_json::from_str(
            r#"{"id":"1","clusterId":"c","alias":null,"namespace":"ns","workloadType":"service",
                "workloadName":"web","forwards":[{"localPort":1,"remotePort":2}],"localAddress":null}"#,
        )
        .unwrap();
        assert_eq!(svc.alias, None);
        assert_eq!(svc.local_address, None);
        assert!(
            !svc.forwards[0].id.is_empty(),
            "missing forward ids are generated"
        );
    }

    #[test]
    fn labels_and_addresses() {
        assert_eq!(service(Some("db"), "postgres").label(), "db");
        assert_eq!(service(None, "postgres").label(), "postgres");
        let mut svc = service(None, "web");
        assert_eq!(svc.bind_address(), "localhost");
        svc.local_address = Some("127.0.1.1".into());
        assert_eq!(svc.bind_address(), "127.0.1.1");
    }

    #[test]
    fn normalization_drops_empty_values() {
        let mut svc = service(Some("  "), " web ");
        svc.local_address = Some("".into());
        let svc = svc.normalized();
        assert_eq!(svc.alias, None);
        assert_eq!(svc.local_address, None);
        assert_eq!(svc.workload_name, "web");

        let config = ClusterConfig {
            storing_method: StoringMethod::Content,
            path: Some("".into()),
            content: Some("\n\napiVersion: v1\n  \n".into()),
            current_context: Some("".into()),
        }
        .normalized();
        assert_eq!(config.path, None);
        assert_eq!(config.current_context, None);
        assert_eq!(config.content.as_deref(), Some("apiVersion: v1\n"));
    }

    #[test]
    fn query_is_case_insensitive() {
        let svc = service(Some("Main DB"), "postgres-0");
        assert!(svc.matches_query("main"));
        assert!(svc.matches_query("POSTGRES"));
        assert!(svc.matches_query(""));
        assert!(!svc.matches_query("redis"));
    }

    #[test]
    fn deleting_a_cluster_deletes_its_services_and_cleanup_removes_orphans() {
        let mut state = AppState::default();
        state.clusters.push(Cluster {
            id: "c".into(),
            name: "c".into(),
            folded: false,
            config: ClusterConfig::default(),
        });
        state.services.push(service(None, "a"));
        let mut orphan = service(None, "b");
        orphan.cluster_id = "missing".into();
        state.services.push(orphan);

        state.cleanup();
        assert_eq!(state.services.len(), 1);

        state.delete_cluster("c");
        assert!(state.clusters.is_empty());
        assert!(state.services.is_empty());
    }

    #[test]
    fn clusters_are_sorted_by_name() {
        let mut state = AppState::default();
        for name in ["beta", "Alpha", "alpha", "Gamma"] {
            state.clusters.push(Cluster {
                id: new_id(),
                name: name.into(),
                folded: false,
                config: ClusterConfig::default(),
            });
        }
        let names: Vec<&str> = state
            .sorted_clusters()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, ["Alpha", "alpha", "beta", "Gamma"]);
    }
}
