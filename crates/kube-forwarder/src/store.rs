//! Application state shared by all views.

use std::collections::HashMap;
use std::path::PathBuf;

use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{App, Context, EventEmitter, Global, Task};
use kf_core::KfError;
use kf_core::export::ExportedService;
use kf_core::forward::{ForwardEvent, ForwardManager, ForwardSpec};
use kf_core::model::{AppState, Cluster, Service, new_id};
use kf_core::storage::Storage;
use kf_core::validate::{self, ValidationError};

/// Tokio runtime used for Kubernetes requests and port forwarding (GPUI has its own
/// executors, kube-rs needs Tokio).
#[derive(Clone)]
pub struct Tokio(pub tokio::runtime::Handle);

impl Global for Tokio {}

/// Runs a future on the Tokio runtime and returns a future that can be awaited from GPUI.
pub fn run_in_tokio<T, F>(
    cx: &App,
    future: F,
) -> impl Future<Output = Result<T, KfError>> + use<T, F>
where
    T: Send + 'static,
    F: Future<Output = Result<T, KfError>> + Send + 'static,
{
    let task = cx.global::<Tokio>().0.spawn(future);
    async move {
        task.await.unwrap_or_else(|error| {
            Err(KfError::new("A background task failed.").with_details(error.to_string()))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortPhase {
    Connecting,
    Connected,
}

/// A local port in use (the `Connections` module of the original store).
#[derive(Debug, Clone)]
pub struct PortConnection {
    pub service_id: String,
    pub phase: PortPhase,
    pub http: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Not running, all ports are free.
    Ready,
    Connecting,
    Connected,
    /// Some ports are used by another resource.
    Blocked,
}

impl ServiceStatus {
    /// Resources can only be edited and deleted while they are not running.
    pub fn can_modify(self) -> bool {
        matches!(self, ServiceStatus::Ready | ServiceStatus::Blocked)
    }
}

pub enum StoreEvent {
    /// An error that is not tied to a user action (for example saving failed).
    Error(KfError),
}

pub enum StartupNotice {
    Imported { from: PathBuf, clusters: usize },
    Warning(String),
}

pub struct AppStore {
    pub state: AppState,
    storage: Storage,
    connections: HashMap<(String, u16), PortConnection>,
    /// Last failed connection of running resources (port and error).
    issues: HashMap<String, (u16, KfError)>,
    /// Start generation of every running resource, guards against stale results.
    sessions: HashMap<String, u64>,
    next_session: u64,
    forwards: ForwardManager,
    pub startup_notice: Option<StartupNotice>,
    _events: Task<()>,
}

impl EventEmitter<StoreEvent> for AppStore {}

impl AppStore {
    pub fn new(storage: Storage, cx: &mut Context<Self>) -> Self {
        let outcome = storage.load();
        let startup_notice = match (outcome.imported_from, outcome.warning) {
            (_, Some(warning)) => Some(StartupNotice::Warning(warning)),
            (Some(from), None) => Some(StartupNotice::Imported {
                from,
                clusters: outcome.state.clusters.len(),
            }),
            (None, None) => None,
        };

        let (sender, mut receiver) = mpsc::unbounded();
        let forwards = ForwardManager::new(cx.global::<Tokio>().0.clone(), sender);
        let events = cx.spawn(async move |this, cx| {
            while let Some(event) = receiver.next().await {
                if this
                    .update(cx, |store, cx| store.on_forward_event(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            state: outcome.state,
            storage,
            connections: HashMap::new(),
            issues: HashMap::new(),
            sessions: HashMap::new(),
            next_session: 1,
            forwards,
            startup_notice,
            _events: events,
        }
    }

    fn commit(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.storage.save(&self.state) {
            tracing::error!(error = %error.full_text(), "failed to save the state");
            cx.emit(StoreEvent::Error(error));
        }
        cx.notify();
    }

    // ---------------------------------------------------------------------------------
    // Clusters
    // ---------------------------------------------------------------------------------

    pub fn create_cluster(
        &mut self,
        mut cluster: Cluster,
        cx: &mut Context<Self>,
    ) -> Result<Cluster, Vec<ValidationError>> {
        cluster.id = new_id();
        cluster.name = cluster.name.trim().to_string();
        cluster.config = cluster.config.normalized();
        let errors = validate::validate_cluster(&cluster);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.state.clusters.push(cluster.clone());
        self.commit(cx);
        Ok(cluster)
    }

    pub fn update_cluster(
        &mut self,
        mut cluster: Cluster,
        cx: &mut Context<Self>,
    ) -> Result<(), Vec<ValidationError>> {
        cluster.name = cluster.name.trim().to_string();
        cluster.config = cluster.config.normalized();
        let errors = validate::validate_cluster(&cluster);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.state.upsert_cluster(cluster);
        self.commit(cx);
        Ok(())
    }

    /// Creates a cluster together with resources of an import.
    pub fn import_cluster(
        &mut self,
        cluster: Cluster,
        services: &[ExportedService],
        cx: &mut Context<Self>,
    ) -> Result<Cluster, Vec<ValidationError>> {
        let cluster = self.create_cluster(cluster, cx)?;
        for service in services {
            self.state.services.push(service.to_service(&cluster.id));
        }
        self.commit(cx);
        Ok(cluster)
    }

    pub fn add_clusters(&mut self, clusters: Vec<Cluster>, cx: &mut Context<Self>) {
        for mut cluster in clusters {
            cluster.id = new_id();
            cluster.config = cluster.config.normalized();
            self.state.clusters.push(cluster);
        }
        self.commit(cx);
    }

    pub fn delete_cluster(&mut self, id: &str, cx: &mut Context<Self>) {
        let services: Vec<String> = self.state.services_of(id).map(|s| s.id.clone()).collect();
        for service_id in services {
            self.stop_service(&service_id, cx);
        }
        self.state.delete_cluster(id);
        self.commit(cx);
    }

    pub fn set_folded(&mut self, ids: &[String], folded: bool, cx: &mut Context<Self>) {
        for id in ids {
            if let Some(cluster) = self.state.cluster_mut(id) {
                cluster.folded = folded;
            }
        }
        self.commit(cx);
    }

    // ---------------------------------------------------------------------------------
    // Resources
    // ---------------------------------------------------------------------------------

    pub fn create_service(
        &mut self,
        mut service: Service,
        cx: &mut Context<Self>,
    ) -> Result<Service, Vec<ValidationError>> {
        service.id = new_id();
        let service = service.normalized();
        let errors = validate::validate_service(&service, &self.state);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.state.services.push(service.clone());
        self.commit(cx);
        Ok(service)
    }

    pub fn update_service(
        &mut self,
        service: Service,
        cx: &mut Context<Self>,
    ) -> Result<(), Vec<ValidationError>> {
        let service = service.normalized();
        let errors = validate::validate_service(&service, &self.state);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.state.upsert_service(service);
        self.commit(cx);
        Ok(())
    }

    pub fn delete_service(&mut self, id: &str, cx: &mut Context<Self>) {
        self.stop_service(id, cx);
        self.state.delete_service(id);
        self.commit(cx);
    }

    // ---------------------------------------------------------------------------------
    // Forwarding
    // ---------------------------------------------------------------------------------

    fn port_states<'a>(
        &'a self,
        service: &'a Service,
    ) -> impl Iterator<Item = Option<&'a PortConnection>> + 'a {
        let address = service.bind_address().to_string();
        service
            .forwards
            .iter()
            .map(move |forward| self.connections.get(&(address.clone(), forward.local_port)))
    }

    pub fn status(&self, service: &Service) -> ServiceStatus {
        let states: Vec<Option<&PortConnection>> = self.port_states(service).collect();
        if states.iter().all(Option::is_none) {
            return ServiceStatus::Ready;
        }
        let own = states
            .iter()
            .all(|state| state.is_some_and(|state| state.service_id == service.id));
        if own {
            let connected = states
                .iter()
                .all(|state| state.is_some_and(|state| state.phase == PortPhase::Connected));
            if connected {
                ServiceStatus::Connected
            } else {
                ServiceStatus::Connecting
            }
        } else {
            ServiceStatus::Blocked
        }
    }

    pub fn is_http(&self, service: &Service, port: u16) -> bool {
        self.connections
            .get(&(service.bind_address().to_string(), port))
            .is_some_and(|connection| connection.service_id == service.id && connection.http)
    }

    pub fn issue(&self, service_id: &str) -> Option<&(u16, KfError)> {
        self.issues.get(service_id)
    }

    /// Ports of a resource used by other resources, for the "blocked" explanation.
    pub fn blocking_ports(&self, service: &Service) -> Vec<(u16, String)> {
        let address = service.bind_address().to_string();
        service
            .forwards
            .iter()
            .filter_map(|forward| {
                let connection = self
                    .connections
                    .get(&(address.clone(), forward.local_port))?;
                if connection.service_id == service.id {
                    return None;
                }
                let name = self
                    .state
                    .service(&connection.service_id)
                    .map(|other| other.label().to_string())
                    .unwrap_or_else(|| format!("Service(serviceId:{})", connection.service_id));
                Some((forward.local_port, name))
            })
            .collect()
    }

    pub fn start_service(
        &mut self,
        service_id: &str,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), KfError>> {
        let Some(service) = self.state.service(service_id).cloned() else {
            return Task::ready(Err(KfError::new("The resource doesn't exist.")));
        };
        let Some(cluster) = self.state.cluster(&service.cluster_id).cloned() else {
            return Task::ready(Err(KfError::new(format!(
                "Cluster(id={}) doesn't exist",
                service.cluster_id
            ))));
        };
        let address = service.bind_address().to_string();
        for forward in &service.forwards {
            if self
                .connections
                .contains_key(&(address.clone(), forward.local_port))
            {
                return Task::ready(Err(KfError::new(format!(
                    "Port {address}:{} is busy.",
                    forward.local_port
                ))));
            }
        }

        let session = self.next_session;
        self.next_session += 1;
        self.sessions.insert(service.id.clone(), session);
        self.issues.remove(&service.id);
        for forward in &service.forwards {
            self.connections.insert(
                (address.clone(), forward.local_port),
                PortConnection {
                    service_id: service.id.clone(),
                    phase: PortPhase::Connecting,
                    http: false,
                },
            );
        }
        cx.notify();

        let start = self
            .forwards
            .start(ForwardSpec::new(&service, &cluster.config));
        cx.spawn(async move |this, cx| {
            let result = start.await;
            this.update(cx, |store, cx| {
                if store.sessions.get(&service.id) != Some(&session) {
                    return; // Stopped (and maybe started again) in the meantime.
                }
                match &result {
                    Ok(started) => {
                        for port in started {
                            tracing::info!(
                                service = %service.label(),
                                local = port.local,
                                pod = %port.pod,
                                target = port.target_port,
                                "forwarding"
                            );
                        }
                        for forward in &service.forwards {
                            if let Some(connection) = store
                                .connections
                                .get_mut(&(address.clone(), forward.local_port))
                                && connection.service_id == service.id
                            {
                                connection.phase = PortPhase::Connected;
                            }
                        }
                    }
                    Err(_) => {
                        store.sessions.remove(&service.id);
                        store.clear_connections(&service.id);
                    }
                }
                cx.notify();
            })
            .ok();
            result.map(|_| ())
        })
    }

    pub fn stop_service(&mut self, service_id: &str, cx: &mut Context<Self>) {
        self.forwards.stop(service_id);
        self.sessions.remove(service_id);
        self.issues.remove(service_id);
        self.clear_connections(service_id);
        cx.notify();
    }

    pub fn stop_all(&mut self) {
        self.forwards.stop_all();
        self.sessions.clear();
        self.connections.clear();
    }

    fn clear_connections(&mut self, service_id: &str) {
        self.connections
            .retain(|_, connection| connection.service_id != service_id);
    }

    fn on_forward_event(&mut self, event: ForwardEvent, cx: &mut Context<Self>) {
        match event {
            ForwardEvent::HttpDetected {
                service_id,
                local_port,
            } => {
                for ((_, port), connection) in self.connections.iter_mut() {
                    if *port == local_port && connection.service_id == service_id {
                        connection.http = true;
                    }
                }
            }
            ForwardEvent::ConnectionFailed {
                service_id,
                local_port,
                error,
            } => {
                if self.sessions.contains_key(&service_id) {
                    self.issues.insert(service_id, (local_port, error));
                }
            }
            ForwardEvent::ConnectionRecovered { service_id, .. } => {
                self.issues.remove(&service_id);
            }
        }
        cx.notify();
    }
}
