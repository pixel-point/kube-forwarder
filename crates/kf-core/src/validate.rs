//! Validation of clusters and resources before they are saved.

use crate::model::{AppState, Cluster, Service, StoringMethod};

pub const REQUIRED: &str = "Field is required.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Field name as used in the JSON format (`name`, `path`, `forwards`, ...).
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

/// Returns the first error message of a field.
pub fn error_for<'a>(errors: &'a [ValidationError], field: &str) -> Option<&'a str> {
    errors
        .iter()
        .find(|error| error.field == field)
        .map(|error| error.message.as_str())
}

fn blank(value: Option<&str>) -> bool {
    value.is_none_or(|value| value.trim().is_empty())
}

pub fn validate_cluster(cluster: &Cluster) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    if cluster.name.trim().is_empty() {
        errors.push(ValidationError::new("name", REQUIRED));
    }
    let config = &cluster.config;
    match config.storing_method {
        StoringMethod::Path => {
            if blank(config.path.as_deref()) {
                errors.push(ValidationError::new("path", REQUIRED));
            }
            if blank(config.current_context.as_deref()) {
                errors.push(ValidationError::new("currentContext", REQUIRED));
            }
        }
        StoringMethod::Content => {
            if blank(config.content.as_deref()) {
                errors.push(ValidationError::new("content", REQUIRED));
            }
        }
    }
    errors
}

pub fn validate_service(service: &Service, state: &AppState) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if service.cluster_id.is_empty() {
        errors.push(ValidationError::new("clusterId", REQUIRED));
    } else if state.cluster(&service.cluster_id).is_none() {
        errors.push(ValidationError::new(
            "clusterId",
            "The cluster doesn't exist.",
        ));
    }
    if service.namespace.trim().is_empty() {
        errors.push(ValidationError::new("namespace", REQUIRED));
    }
    if service.workload_name.trim().is_empty() {
        errors.push(ValidationError::new("workloadName", REQUIRED));
    }

    if service.forwards.is_empty() {
        errors.push(ValidationError::new(
            "forwards",
            "Add at least one port to forward.",
        ));
    }
    for forward in &service.forwards {
        if forward.local_port == 0 || forward.remote_port == 0 {
            errors.push(ValidationError::new(
                "forwards",
                "Ports must be between 1 and 65535.",
            ));
            break;
        }
    }
    let mut seen = Vec::new();
    for forward in &service.forwards {
        if seen.contains(&forward.local_port) {
            errors.push(ValidationError::new(
                "forwards",
                format!("Local port {} is used more than once.", forward.local_port),
            ));
            break;
        }
        seen.push(forward.local_port);
    }

    if let Some(address) = &service.local_address {
        let address = address.trim();
        if address
            .chars()
            .any(|c| c.is_whitespace() || c == '/' || c == '\\')
        {
            errors.push(ValidationError::new("localAddress", "Invalid address."));
        }
    }

    errors
}

/// Parses a port typed into a form field.
pub fn parse_port(value: &str) -> Result<u16, &'static str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(REQUIRED);
    }
    let number: i64 = value.parse().map_err(|_| "Field must be an integer.")?;
    if (1..=65535).contains(&number) {
        Ok(number as u16)
    } else {
        Err("Port must be between 1 and 65535.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ClusterConfig, Forward, WorkloadType, new_id};

    fn cluster(config: ClusterConfig) -> Cluster {
        Cluster {
            id: "c1".into(),
            name: "local".into(),
            folded: false,
            config,
        }
    }

    fn service() -> Service {
        Service {
            id: new_id(),
            cluster_id: "c1".into(),
            alias: None,
            namespace: "default".into(),
            workload_type: WorkloadType::Deployment,
            workload_name: "web".into(),
            forwards: vec![Forward::new(8080, 80)],
            local_address: None,
        }
    }

    #[test]
    fn cluster_requires_fields_of_its_storing_method() {
        let errors = validate_cluster(&cluster(ClusterConfig {
            storing_method: StoringMethod::Path,
            ..Default::default()
        }));
        assert_eq!(error_for(&errors, "path"), Some(REQUIRED));
        assert_eq!(error_for(&errors, "currentContext"), Some(REQUIRED));
        assert_eq!(error_for(&errors, "content"), None);

        let errors = validate_cluster(&cluster(ClusterConfig::default()));
        assert_eq!(error_for(&errors, "content"), Some(REQUIRED));

        assert!(validate_cluster(&cluster(ClusterConfig::from_path("/a", "ctx"))).is_empty());

        let mut unnamed = cluster(ClusterConfig::from_content("a: b", None));
        unnamed.name = "  ".into();
        assert_eq!(
            error_for(&validate_cluster(&unnamed), "name"),
            Some(REQUIRED)
        );
    }

    #[test]
    fn service_validation() {
        let mut state = AppState::default();
        state.clusters.push(cluster(ClusterConfig::default()));

        assert!(validate_service(&service(), &state).is_empty());

        let mut svc = service();
        svc.namespace.clear();
        svc.workload_name = " ".into();
        svc.forwards.clear();
        svc.cluster_id = "missing".into();
        let errors = validate_service(&svc, &state);
        assert!(error_for(&errors, "namespace").is_some());
        assert!(error_for(&errors, "workloadName").is_some());
        assert!(error_for(&errors, "forwards").is_some());
        assert!(error_for(&errors, "clusterId").is_some());

        let mut svc = service();
        svc.forwards.push(Forward::new(8080, 81));
        assert_eq!(
            error_for(&validate_service(&svc, &state), "forwards"),
            Some("Local port 8080 is used more than once.")
        );

        let mut svc = service();
        svc.local_address = Some("my host".into());
        assert!(error_for(&validate_service(&svc, &state), "localAddress").is_some());
        svc.local_address = Some("::1".into());
        assert!(validate_service(&svc, &state).is_empty());
    }

    #[test]
    fn ports_are_parsed_like_the_original_form() {
        assert_eq!(parse_port(" 8080 "), Ok(8080));
        assert_eq!(parse_port(""), Err(REQUIRED));
        assert_eq!(parse_port("80a"), Err("Field must be an integer."));
        assert!(parse_port("0").is_err());
        assert!(parse_port("65536").is_err());
        assert_eq!(parse_port("65535"), Ok(65535));
    }
}
