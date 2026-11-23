//! Port forwarding engine.
//!
//! For every forwarded port a local TCP listener is started. Each accepted connection
//! opens its own port-forward stream to the pod (like the original application and
//! `kubectl port-forward`), so a broken connection never takes the listener down.
//! When a stream can't be opened, the pod behind the resource is looked up again: pods of
//! deployments and services are replaced during rollouts, and forwarding keeps working.

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use k8s_openapi::api::core::v1::Pod;
use kube::Client;
use kube::api::{Api, Portforwarder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

use crate::error::{KfError, Result, bind_error, error_chain};
use crate::k8s::{self, with_timeout};
use crate::kubeconfig;
use crate::model::{ClusterConfig, Service, WorkloadType};

const STREAM_TIMEOUT: Duration = Duration::from_secs(30);
const HTTP_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortPair {
    pub local: u16,
    pub remote: u16,
}

/// Everything needed to forward the ports of one resource.
#[derive(Debug, Clone)]
pub struct ForwardSpec {
    pub service_id: String,
    pub cluster: ClusterConfig,
    pub namespace: String,
    pub kind: WorkloadType,
    pub name: String,
    /// Local address to listen on (`localhost` listens on 127.0.0.1 and ::1).
    pub address: String,
    pub ports: Vec<PortPair>,
}

impl ForwardSpec {
    pub fn new(service: &Service, cluster: &ClusterConfig) -> Self {
        Self {
            service_id: service.id.clone(),
            cluster: cluster.clone(),
            namespace: service.namespace.clone(),
            kind: service.workload_type,
            name: service.workload_name.clone(),
            address: service.bind_address().to_string(),
            ports: service
                .forwards
                .iter()
                .map(|forward| PortPair {
                    local: forward.local_port,
                    remote: forward.remote_port,
                })
                .collect(),
        }
    }
}

/// Events emitted while forwarding is running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardEvent {
    /// The local port answers HTTP requests.
    HttpDetected { service_id: String, local_port: u16 },
    /// A connection could not be forwarded. The port keeps listening.
    ConnectionFailed {
        service_id: String,
        local_port: u16,
        error: KfError,
    },
    /// A connection was forwarded again after a failure.
    ConnectionRecovered { service_id: String, local_port: u16 },
}

/// A port that listens locally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedPort {
    pub local: u16,
    pub remote: u16,
    pub pod: String,
    pub target_port: u16,
    pub addresses: Vec<SocketAddr>,
}

struct Session {
    id: u64,
    cancel: CancellationToken,
}

/// Starts and stops forwarding sessions (one per resource) on a Tokio runtime.
#[derive(Clone)]
pub struct ForwardManager {
    runtime: tokio::runtime::Handle,
    events: UnboundedSender<ForwardEvent>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    next_id: Arc<AtomicU64>,
}

impl ForwardManager {
    pub fn new(runtime: tokio::runtime::Handle, events: UnboundedSender<ForwardEvent>) -> Self {
        Self {
            runtime,
            events,
            sessions: Arc::default(),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Starts forwarding all ports of a resource.
    ///
    /// The returned future resolves once every port listens locally, or with the first
    /// error (nothing keeps running in that case). It can be awaited from any executor.
    pub fn start(
        &self,
        spec: ForwardSpec,
    ) -> impl Future<Output = Result<Vec<StartedPort>>> + Send + 'static {
        let cancel = CancellationToken::new();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        if let Some(previous) = self.sessions.lock().unwrap().insert(
            spec.service_id.clone(),
            Session {
                id,
                cancel: cancel.clone(),
            },
        ) {
            previous.cancel.cancel();
        }

        let manager = self.clone();
        let task = self.runtime.spawn(async move {
            let service_id = spec.service_id.clone();
            let result = tokio::select! {
                _ = cancel.cancelled() => Err(KfError::cancelled()),
                result = start_session(spec, cancel.clone(), manager.events.clone()) => result,
            };
            if result.is_err() {
                cancel.cancel();
                manager.forget(&service_id, id);
            }
            result
        });

        async move {
            task.await.unwrap_or_else(|error| {
                Err(KfError::new("Port forwarding stopped unexpectedly.")
                    .with_details(error.to_string()))
            })
        }
    }

    /// Stops forwarding of a resource (also cancels a start in progress).
    pub fn stop(&self, service_id: &str) {
        if let Some(session) = self.sessions.lock().unwrap().remove(service_id) {
            session.cancel.cancel();
        }
    }

    pub fn stop_all(&self) {
        for (_, session) in self.sessions.lock().unwrap().drain() {
            session.cancel.cancel();
        }
    }

    pub fn is_running(&self, service_id: &str) -> bool {
        self.sessions.lock().unwrap().contains_key(service_id)
    }

    fn forget(&self, service_id: &str, id: u64) {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions
            .get(service_id)
            .is_some_and(|session| session.id == id)
        {
            sessions.remove(service_id);
        }
    }
}

/// The pod (and its ports) connections are forwarded to. Replaced when the pod goes away.
struct Targets {
    generation: u64,
    pod: String,
    /// Resource port → pod port.
    ports: HashMap<u16, u16>,
}

struct SessionShared {
    service_id: String,
    client: Client,
    namespace: String,
    kind: WorkloadType,
    name: String,
    remote_ports: Vec<u16>,
    targets: tokio::sync::Mutex<Targets>,
    events: UnboundedSender<ForwardEvent>,
}

impl SessionShared {
    async fn resolve(&self, generation: u64) -> Result<Targets> {
        let resolved = with_timeout(k8s::resolve_pod(
            &self.client,
            &self.namespace,
            self.kind,
            &self.name,
        ))
        .await?;
        let mut ports = HashMap::new();
        for remote in &self.remote_ports {
            ports.insert(*remote, resolved.target_port(*remote)?);
        }
        Ok(Targets {
            generation,
            pod: resolved.pod_name,
            ports,
        })
    }

    async fn current(&self, remote: u16) -> (u64, String, u16) {
        let targets = self.targets.lock().await;
        let port = targets.ports.get(&remote).copied().unwrap_or(remote);
        (targets.generation, targets.pod.clone(), port)
    }

    /// Looks the pod up again, unless another connection already did it.
    async fn refresh(&self, seen_generation: u64) -> Result<()> {
        let mut targets = self.targets.lock().await;
        if targets.generation != seen_generation {
            return Ok(());
        }
        let refreshed = self.resolve(seen_generation + 1).await?;
        if refreshed.pod != targets.pod {
            tracing::info!(service = %self.service_id, from = %targets.pod, to = %refreshed.pod, "forwarding to another pod");
        }
        *targets = refreshed;
        Ok(())
    }

    fn emit(&self, event: ForwardEvent) {
        let _ = self.events.unbounded_send(event);
    }
}

async fn start_session(
    spec: ForwardSpec,
    cancel: CancellationToken,
    events: UnboundedSender<ForwardEvent>,
) -> Result<Vec<StartedPort>> {
    let client = with_timeout(kubeconfig::client_for(&spec.cluster)).await?;
    let shared = SessionShared {
        service_id: spec.service_id.clone(),
        client,
        namespace: spec.namespace.clone(),
        kind: spec.kind,
        name: spec.name.clone(),
        remote_ports: spec.ports.iter().map(|pair| pair.remote).collect(),
        targets: tokio::sync::Mutex::new(Targets {
            generation: 0,
            pod: String::new(),
            ports: HashMap::new(),
        }),
        events,
    };
    // Fails early when the resource doesn't exist or has no running pods.
    let targets = shared.resolve(0).await?;
    let pod = targets.pod.clone();
    let target_ports = targets.ports.clone();
    *shared.targets.lock().await = targets;
    let shared = Arc::new(shared);

    // Bind every port before accepting anything, so a failure leaves nothing running.
    let mut bound = Vec::new();
    for pair in &spec.ports {
        bound.push((*pair, bind(&spec.address, pair.local).await?));
    }

    let mut started = Vec::new();
    for (pair, listeners) in bound {
        let addresses: Vec<SocketAddr> = listeners
            .iter()
            .filter_map(|listener| listener.local_addr().ok())
            .collect();
        let port = Arc::new(PortState {
            local: pair.local,
            remote: pair.remote,
            failing: AtomicBool::new(false),
        });
        for listener in listeners {
            tokio::spawn(accept_loop(
                listener,
                shared.clone(),
                port.clone(),
                cancel.clone(),
            ));
        }
        if let Some(address) = addresses.first().copied() {
            let shared = shared.clone();
            let host = host_header(&spec.address, pair.local);
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let detected = tokio::select! {
                    _ = cancel.cancelled() => false,
                    detected = probe_http(address, &host) => detected,
                };
                if detected {
                    shared.emit(ForwardEvent::HttpDetected {
                        service_id: shared.service_id.clone(),
                        local_port: pair.local,
                    });
                }
            });
        }
        started.push(StartedPort {
            local: pair.local,
            remote: pair.remote,
            pod: pod.clone(),
            target_port: target_ports
                .get(&pair.remote)
                .copied()
                .unwrap_or(pair.remote),
            addresses,
        });
    }
    Ok(started)
}

struct PortState {
    local: u16,
    remote: u16,
    failing: AtomicBool,
}

async fn accept_loop(
    listener: TcpListener,
    shared: Arc<SessionShared>,
    port: Arc<PortState>,
    cancel: CancellationToken,
) {
    loop {
        let accepted = tokio::select! {
            _ = cancel.cancelled() => break,
            accepted = listener.accept() => accepted,
        };
        match accepted {
            Ok((socket, _)) => {
                tokio::spawn(handle_connection(
                    socket,
                    shared.clone(),
                    port.clone(),
                    cancel.clone(),
                ));
            }
            Err(error) => {
                // For example too many open files. Don't spin.
                tracing::warn!(%error, "failed to accept a connection");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    shared: Arc<SessionShared>,
    port: Arc<PortState>,
    cancel: CancellationToken,
) {
    let _ = socket.set_nodelay(true);
    let result = tokio::select! {
        _ = cancel.cancelled() => return,
        result = forward_connection(&mut socket, &shared, &port) => result,
    };
    match result {
        Ok(()) => {
            if port.failing.swap(false, Ordering::Relaxed) {
                shared.emit(ForwardEvent::ConnectionRecovered {
                    service_id: shared.service_id.clone(),
                    local_port: port.local,
                });
            }
        }
        Err(error) => {
            tracing::warn!(service = %shared.service_id, port = port.local, error = %error.full_text(), "connection failed");
            port.failing.store(true, Ordering::Relaxed);
            shared.emit(ForwardEvent::ConnectionFailed {
                service_id: shared.service_id.clone(),
                local_port: port.local,
                error,
            });
        }
    }
}

async fn forward_connection(
    socket: &mut TcpStream,
    shared: &SessionShared,
    port: &PortState,
) -> Result<()> {
    let (mut forwarder, pod, target_port) = open_stream(shared, port.remote).await?;
    let upstream = forwarder
        .take_stream(target_port)
        .ok_or_else(|| KfError::new("Port forward stream is not available."))?;
    let pod_error = forwarder.take_error(target_port);

    relay(socket, upstream).await;
    let _ = tokio::time::timeout(Duration::from_secs(5), forwarder.join()).await;

    if let Some(pod_error) = pod_error
        && let Ok(Some(message)) = tokio::time::timeout(Duration::from_secs(1), pod_error).await
    {
        return Err(KfError::new(format!(
            "Error forwarding port {target_port} to pod \"{pod}\"."
        ))
        .with_details(message));
    }
    Ok(())
}

/// Copies data in both directions until the pod side closes.
///
/// The WebSocket port-forward protocol can't half-close a stream: closing the direction
/// to the pod drops everything the pod still sends. So when the local client stops
/// sending (for example `HTTP/1.0` clients or `nc -N`), the answer of the pod is still
/// relayed while it keeps arriving, and the connection is closed after [`LINGER`] of
/// silence. Reset connections are normal and not reported.
async fn relay(socket: &mut TcpStream, upstream: tokio::io::DuplexStream) {
    let (mut client_read, mut client_write) = socket.split();
    let (mut pod_read, mut pod_write) = tokio::io::split(upstream);
    let mut client_buf = vec![0u8; 32 * 1024];
    let mut pod_buf = vec![0u8; 32 * 1024];
    let mut client_done = false;

    loop {
        tokio::select! {
            read = client_read.read(&mut client_buf), if !client_done => match read {
                Ok(0) | Err(_) => client_done = true,
                Ok(n) => {
                    if pod_write.write_all(&client_buf[..n]).await.is_err() {
                        break;
                    }
                }
            },
            read = pod_read.read(&mut pod_buf) => match read {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if client_write.write_all(&pod_buf[..n]).await.is_err() {
                        break;
                    }
                }
            },
            _ = tokio::time::sleep(LINGER), if client_done => break,
        }
    }
    let _ = client_write.shutdown().await;
}

/// How long the answer of a pod is awaited after the local client stopped sending.
const LINGER: Duration = Duration::from_secs(2);

/// Opens a port-forward stream, looking the pod up again once if that fails.
async fn open_stream(shared: &SessionShared, remote: u16) -> Result<(Portforwarder, String, u16)> {
    let pods: Api<Pod> = Api::namespaced(shared.client.clone(), &shared.namespace);
    let mut last_error = None;
    for attempt in 0..2 {
        let (generation, pod, target_port) = shared.current(remote).await;
        match tokio::time::timeout(STREAM_TIMEOUT, pods.portforward(&pod, &[target_port])).await {
            Ok(Ok(forwarder)) => return Ok((forwarder, pod, target_port)),
            Ok(Err(error)) => {
                last_error = Some(
                    KfError::new(format!("Failed to connect to pod \"{pod}\"."))
                        .with_details(error_chain(&error)),
                );
            }
            Err(_) => {
                last_error = Some(KfError::new(format!(
                    "Timed out while connecting to pod \"{pod}\"."
                )));
            }
        }
        if attempt == 0 {
            // The pod may have been replaced (rollout, eviction, ...).
            shared.refresh(generation).await?;
        }
    }
    Err(last_error.expect("at least one attempt was made"))
}

async fn bind(address: &str, port: u16) -> Result<Vec<TcpListener>> {
    let localhost = address.eq_ignore_ascii_case("localhost");
    let candidates: Vec<SocketAddr> = if localhost {
        vec![
            SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
            SocketAddr::from((Ipv6Addr::LOCALHOST, port)),
        ]
    } else {
        let mut resolved: Vec<SocketAddr> = tokio::net::lookup_host((address, port))
            .await
            .map_err(|error| {
                KfError::new(format!("Can't resolve the local address \"{address}\"."))
                    .with_details(error.to_string())
            })?
            .collect();
        resolved.dedup();
        resolved
    };

    let mut listeners = Vec::new();
    let mut first_error = None;
    for (index, candidate) in candidates.iter().enumerate() {
        match bind_with_retry(*candidate).await {
            Ok(listener) => listeners.push(listener),
            // `localhost` must be reachable over IPv4, IPv6 is best effort (like kubectl).
            Err(error) if localhost && index == 0 => return Err(bind_error(&error, address, port)),
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    match (listeners.is_empty(), first_error) {
        (true, Some(error)) => Err(bind_error(&error, address, port)),
        (true, None) => Err(KfError::new(format!(
            "Can't resolve the local address \"{address}\"."
        ))),
        _ => Ok(listeners),
    }
}

async fn bind_with_retry(address: SocketAddr) -> std::io::Result<TcpListener> {
    // A listener of a session that was stopped a moment ago may still be closing.
    let mut attempt = 0;
    loop {
        match TcpListener::bind(address).await {
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse && attempt < 4 => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            result => return result,
        }
    }
}

fn host_header(address: &str, port: u16) -> String {
    if address.contains(':') && !address.starts_with('[') {
        format!("[{address}]:{port}")
    } else {
        format!("{address}:{port}")
    }
}

/// Sends an `OPTIONS` request (like the original application) and checks whether the
/// answer looks like HTTP.
async fn probe_http(address: SocketAddr, host: &str) -> bool {
    let probe = async {
        let mut stream = TcpStream::connect(address).await.ok()?;
        let request = format!("OPTIONS / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.ok()?;
        let mut head = [0u8; 5];
        stream.read_exact(&mut head).await.ok()?;
        Some(&head == b"HTTP/")
    };
    tokio::time::timeout(HTTP_PROBE_TIMEOUT, probe)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn localhost_binds_ipv4_and_ipv6() {
        let listeners = bind("localhost", 0).await.unwrap();
        assert!(listeners[0].local_addr().unwrap().is_ipv4());
    }

    #[tokio::test]
    async fn busy_ports_are_reported() {
        let taken = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = taken.local_addr().unwrap().port();
        let error = bind("127.0.0.1", port).await.unwrap_err();
        assert_eq!(
            error.message,
            format!("Port 127.0.0.1:{port} is already in use.")
        );
        let error = bind("localhost", port).await.unwrap_err();
        assert_eq!(
            error.message,
            format!("Port localhost:{port} is already in use.")
        );
    }

    #[tokio::test]
    async fn unknown_addresses_are_reported() {
        let error = bind("definitely-not-a-host.invalid", 1234)
            .await
            .unwrap_err();
        assert!(error.message.contains("Can't resolve"), "{error:?}");
    }

    #[tokio::test]
    async fn http_probe() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            let _ = socket.read(&mut buf).await;
            let _ = socket.write_all(b"HTTP/1.1 204 No Content\r\n\r\n").await;
        });
        assert!(probe_http(address, "localhost").await);

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let _ = socket.write_all(b"\x00\x00\x00\x08garbage").await;
        });
        assert!(!probe_http(address, "localhost").await);
    }

    #[test]
    fn host_headers() {
        assert_eq!(host_header("localhost", 80), "localhost:80");
        assert_eq!(host_header("::1", 80), "[::1]:80");
    }
}
