//! Kubernetes API operations used by the application.

use std::fmt::Debug;
use std::time::Duration;

use k8s_openapi::NamespaceResourceScope;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Pod, Service as KubeService};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::Client;
use kube::api::{Api, ListParams};
use serde::de::DeserializeOwned;

use crate::error::{KfError, Result, kube_error};
use crate::kubeconfig;
use crate::model::{ClusterConfig, WorkloadType};

/// Upper bound for API calls (including running exec credential plugins, which may
/// open a browser for an interactive login).
pub const API_TIMEOUT: Duration = Duration::from_secs(90);

pub async fn with_timeout<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
    tokio::time::timeout(API_TIMEOUT, future)
        .await
        .unwrap_or_else(|_| Err(KfError::new("Timed out while waiting for the cluster.")))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionCheck {
    Ok,
    /// Authenticated, but the user is not allowed to list namespaces.
    Restricted(String),
}

/// Connects to the cluster and makes a cheap authenticated request.
pub async fn check_connection(config: &ClusterConfig) -> Result<ConnectionCheck> {
    with_timeout(async {
        let client = kubeconfig::client_for(config).await?;
        let namespaces: Api<Namespace> = Api::all(client);
        match namespaces
            .list_metadata(&ListParams::default().limit(1))
            .await
        {
            Ok(_) => Ok(ConnectionCheck::Ok),
            Err(kube::Error::Api(status)) if status.code == 403 => {
                Ok(ConnectionCheck::Restricted(status.message))
            }
            Err(error) => Err(kube_error(&error, None)),
        }
    })
    .await
}

pub async fn list_namespaces(client: &Client) -> Result<Vec<String>> {
    with_timeout(async {
        let api: Api<Namespace> = Api::all(client.clone());
        let list = api
            .list_metadata(&ListParams::default())
            .await
            .map_err(|error| kube_error(&error, Some("Namespaces")))?;
        let mut names: Vec<String> = list
            .items
            .into_iter()
            .filter_map(|item| item.metadata.name)
            .collect();
        names.sort();
        Ok(names)
    })
    .await
}

pub async fn list_workload_names(
    client: &Client,
    namespace: &str,
    kind: WorkloadType,
) -> Result<Vec<String>> {
    with_timeout(async {
        match kind {
            WorkloadType::Pod => list_names::<Pod>(client, namespace, "Pods").await,
            WorkloadType::Deployment => {
                list_names::<Deployment>(client, namespace, "Deployments").await
            }
            WorkloadType::Service => list_names::<KubeService>(client, namespace, "Services").await,
        }
    })
    .await
}

async fn list_names<K>(client: &Client, namespace: &str, plural: &str) -> Result<Vec<String>>
where
    K: kube::Resource<Scope = NamespaceResourceScope> + Clone + DeserializeOwned + Debug,
    K::DynamicType: Default,
{
    let api: Api<K> = Api::namespaced(client.clone(), namespace);
    let list = api
        .list_metadata(&ListParams::default())
        .await
        .map_err(|error| kube_error(&error, Some(plural)))?;
    let mut names: Vec<String> = list
        .items
        .into_iter()
        .filter_map(|item| item.metadata.name)
        .collect();
    names.sort();
    Ok(names)
}

/// The pod that receives forwarded connections of a resource.
#[derive(Debug, Clone)]
pub struct ResolvedPod {
    pub pod_name: String,
    pod: Pod,
    service: Option<KubeService>,
}

impl ResolvedPod {
    /// Port of the pod for a port configured by the user ("Resource port"). For services
    /// it's the service port, which is mapped to the target port of the pod.
    pub fn target_port(&self, port: u16) -> Result<u16> {
        match &self.service {
            Some(service) => map_service_port(service, port, &self.pod),
            None => Ok(port),
        }
    }
}

/// Finds the pod behind a pod, a deployment or a service.
pub async fn resolve_pod(
    client: &Client,
    namespace: &str,
    kind: WorkloadType,
    name: &str,
) -> Result<ResolvedPod> {
    match kind {
        WorkloadType::Pod => {
            let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
            let pod = pods
                .get(name)
                .await
                .map_err(|error| kube_error(&error, Some(&format!("Pod \"{name}\""))))?;
            let phase = pod
                .status
                .as_ref()
                .and_then(|status| status.phase.clone())
                .unwrap_or_else(|| "Unknown".into());
            if phase != "Running" {
                return Err(KfError::new(format!(
                    "Pod \"{name}\" is not running (status: {phase})."
                )));
            }
            Ok(ResolvedPod {
                pod_name: name.to_string(),
                pod,
                service: None,
            })
        }
        WorkloadType::Deployment => {
            let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            let deployment = deployments
                .get(name)
                .await
                .map_err(|error| kube_error(&error, Some(&format!("Deployment \"{name}\""))))?;
            let selector = deployment
                .spec
                .as_ref()
                .and_then(|spec| label_selector_string(&spec.selector))
                .ok_or_else(|| {
                    KfError::new(format!("Deployment \"{name}\" does not have a selector."))
                })?;
            let pod = find_pod(client, namespace, &selector)
                .await?
                .ok_or_else(|| {
                    KfError::new(format!(
                        "There are no running pods in \"{name}\" deployment."
                    ))
                })?;
            Ok(ResolvedPod {
                pod_name: pod.metadata.name.clone().unwrap_or_default(),
                pod,
                service: None,
            })
        }
        WorkloadType::Service => {
            let services: Api<KubeService> = Api::namespaced(client.clone(), namespace);
            let service = services
                .get(name)
                .await
                .map_err(|error| kube_error(&error, Some(&format!("Service \"{name}\""))))?;
            let selector = service
                .spec
                .as_ref()
                .and_then(|spec| spec.selector.as_ref())
                .filter(|selector| !selector.is_empty())
                .map(|selector| {
                    selector
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .ok_or_else(|| {
                    KfError::new(format!("Service \"{name}\" does not have a selector."))
                })?;
            let pod = find_pod(client, namespace, &selector)
                .await?
                .ok_or_else(|| {
                    KfError::new(format!("There are no running pods in \"{name}\" service."))
                })?;
            Ok(ResolvedPod {
                pod_name: pod.metadata.name.clone().unwrap_or_default(),
                pod,
                service: Some(service),
            })
        }
    }
}

async fn find_pod(client: &Client, namespace: &str, selector: &str) -> Result<Option<Pod>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let list = pods
        .list(&ListParams::default().labels(selector))
        .await
        .map_err(|error| kube_error(&error, Some("Pods")))?;
    Ok(pick_pod(list.items))
}

/// Converts a label selector to its string form (`a=b,c in (d,e),!f`).
pub fn label_selector_string(selector: &LabelSelector) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(labels) = &selector.match_labels {
        for (key, value) in labels {
            parts.push(format!("{key}={value}"));
        }
    }
    if let Some(expressions) = &selector.match_expressions {
        for expression in expressions {
            let values = expression.values.clone().unwrap_or_default().join(",");
            match expression.operator.as_str() {
                "In" => parts.push(format!("{} in ({values})", expression.key)),
                "NotIn" => parts.push(format!("{} notin ({values})", expression.key)),
                "Exists" => parts.push(expression.key.clone()),
                "DoesNotExist" => parts.push(format!("!{}", expression.key)),
                _ => {}
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join(","))
}

/// Picks the pod to forward to: running and ready pods first, pods that are being
/// deleted are skipped.
pub fn pick_pod(pods: Vec<Pod>) -> Option<Pod> {
    fn score(pod: &Pod) -> u8 {
        if pod.metadata.deletion_timestamp.is_some() {
            return 0;
        }
        let Some(status) = pod.status.as_ref() else {
            return 0;
        };
        if status.phase.as_deref() != Some("Running") {
            return 0;
        }
        let ready = status.conditions.as_ref().is_some_and(|conditions| {
            conditions
                .iter()
                .any(|condition| condition.type_ == "Ready" && condition.status == "True")
        });
        if ready { 2 } else { 1 }
    }

    let mut best: Option<(u8, Pod)> = None;
    for pod in pods {
        let pod_score = score(&pod);
        if pod_score > 0
            && best
                .as_ref()
                .is_none_or(|(best_score, _)| pod_score > *best_score)
        {
            best = Some((pod_score, pod));
        }
    }
    best.map(|(_, pod)| pod)
}

/// Maps a service port to the port of a pod (`targetPort`, which may be a named port).
pub fn map_service_port(service: &KubeService, port: u16, pod: &Pod) -> Result<u16> {
    let name = service.metadata.name.clone().unwrap_or_default();
    let ports = service
        .spec
        .as_ref()
        .and_then(|spec| spec.ports.clone())
        .unwrap_or_default();
    let Some(service_port) = ports.iter().find(|item| item.port == i32::from(port)) else {
        let available = ports
            .iter()
            .map(|item| item.port.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(KfError::new(format!(
            "Service \"{name}\" does not have a service port {port}. Available ports: {available}"
        )));
    };

    let to_port = |value: i32| {
        u16::try_from(value).map_err(|_| KfError::new(format!("Invalid target port {value}.")))
    };

    match &service_port.target_port {
        None => Ok(port),
        Some(IntOrString::Int(target)) => to_port(*target),
        Some(IntOrString::String(target)) => {
            if let Ok(number) = target.parse::<u16>() {
                return Ok(number);
            }
            pod.spec
                .iter()
                .flat_map(|spec| spec.containers.iter())
                .flat_map(|container| container.ports.iter().flatten())
                .find(|container_port| container_port.name.as_deref() == Some(target.as_str()))
                .map(|container_port| to_port(container_port.container_port))
                .unwrap_or_else(|| {
                    Err(KfError::new(format!(
                        "Pod \"{}\" does not have a port named \"{target}\" (target port of port {port} of service \"{name}\").",
                        pod.metadata.name.clone().unwrap_or_default()
                    )))
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        Container, ContainerPort, PodCondition, PodSpec, PodStatus, ServicePort, ServiceSpec,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{
        LabelSelectorRequirement, ObjectMeta, Time,
    };
    use std::collections::BTreeMap;

    fn pod(name: &str, phase: &str, ready: bool) -> Pod {
        Pod {
            metadata: ObjectMeta {
                name: Some(name.into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "app".into(),
                    ports: Some(vec![ContainerPort {
                        name: Some("http".into()),
                        container_port: 8080,
                        ..Default::default()
                    }]),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some(phase.into()),
                conditions: Some(vec![PodCondition {
                    type_: "Ready".into(),
                    status: if ready { "True" } else { "False" }.into(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        }
    }

    fn service(ports: Vec<(i32, Option<IntOrString>)>) -> KubeService {
        KubeService {
            metadata: ObjectMeta {
                name: Some("web".into()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                ports: Some(
                    ports
                        .into_iter()
                        .map(|(port, target_port)| ServicePort {
                            port,
                            target_port,
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn maps_service_ports() {
        let pod = pod("p", "Running", true);
        let svc = service(vec![
            (80, Some(IntOrString::String("http".into()))),
            (81, Some(IntOrString::Int(9000))),
            (82, None),
            (83, Some(IntOrString::String("missing".into()))),
        ]);
        assert_eq!(map_service_port(&svc, 80, &pod).unwrap(), 8080);
        assert_eq!(map_service_port(&svc, 81, &pod).unwrap(), 9000);
        assert_eq!(map_service_port(&svc, 82, &pod).unwrap(), 82);
        assert!(
            map_service_port(&svc, 83, &pod)
                .unwrap_err()
                .message
                .contains("named \"missing\"")
        );
        let error = map_service_port(&svc, 1234, &pod).unwrap_err();
        assert_eq!(
            error.message,
            "Service \"web\" does not have a service port 1234. Available ports: 80, 81, 82, 83"
        );
    }

    #[test]
    fn picks_ready_running_pods() {
        let mut terminating = pod("terminating", "Running", true);
        terminating.metadata.deletion_timestamp = Some(Time(Default::default()));
        let picked = pick_pod(vec![
            pod("pending", "Pending", false),
            terminating,
            pod("not-ready", "Running", false),
            pod("ready", "Running", true),
            pod("ready-2", "Running", true),
        ])
        .unwrap();
        assert_eq!(picked.metadata.name.as_deref(), Some("ready"));

        let picked = pick_pod(vec![pod("not-ready", "Running", false)]).unwrap();
        assert_eq!(picked.metadata.name.as_deref(), Some("not-ready"));

        assert!(pick_pod(vec![pod("pending", "Pending", false)]).is_none());
    }

    #[test]
    fn label_selectors() {
        let selector = LabelSelector {
            match_labels: Some(BTreeMap::from([
                ("app".to_string(), "web".to_string()),
                ("tier".to_string(), "front".to_string()),
            ])),
            match_expressions: Some(vec![
                LabelSelectorRequirement {
                    key: "env".into(),
                    operator: "In".into(),
                    values: Some(vec!["dev".into(), "prod".into()]),
                },
                LabelSelectorRequirement {
                    key: "canary".into(),
                    operator: "DoesNotExist".into(),
                    values: None,
                },
            ]),
        };
        assert_eq!(
            label_selector_string(&selector).as_deref(),
            Some("app=web,tier=front,env in (dev,prod),!canary")
        );
        assert_eq!(label_selector_string(&LabelSelector::default()), None);
    }
}
