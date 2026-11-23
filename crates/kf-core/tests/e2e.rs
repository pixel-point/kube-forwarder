//! End-to-end tests of the port forwarding engine against a real cluster.
//!
//! They are ignored by default because they need a disposable cluster with the
//! workloads of `tests/fixtures/e2e.yaml`. `scripts/e2e.sh` creates one with k3d and runs
//! them, or run them manually:
//!
//! ```sh
//! kubectl apply -f crates/kf-core/tests/fixtures/e2e.yaml
//! KF_E2E_KUBECONFIG=/path/to/kubeconfig cargo test -p kf-core --test e2e -- --ignored --test-threads=1
//! ```

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc;
use k8s_openapi::api::core::v1::Pod;
use kf_core::forward::{ForwardEvent, ForwardManager, ForwardSpec, PortPair};
use kf_core::model::{ClusterConfig, WorkloadType};
use kf_core::{k8s, kubeconfig};
use kube::api::{Api, DeleteParams};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const NS: &str = "kf-e2e";

fn cluster() -> ClusterConfig {
    let path = std::env::var("KF_E2E_KUBECONFIG")
        .expect("set KF_E2E_KUBECONFIG to the kube config of a disposable cluster");
    let parsed = kubeconfig::read_file(Path::new(&path)).unwrap();
    ClusterConfig::from_path(path, parsed.current_context.expect("current-context"))
}

fn free_ports<const N: usize>() -> [u16; N] {
    let listeners: Vec<_> = (0..N)
        .map(|_| std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap())
        .collect();
    std::array::from_fn(|i| listeners[i].local_addr().unwrap().port())
}

fn manager() -> (ForwardManager, mpsc::UnboundedReceiver<ForwardEvent>) {
    let (sender, receiver) = mpsc::unbounded();
    (
        ForwardManager::new(tokio::runtime::Handle::current(), sender),
        receiver,
    )
}

fn spec(id: &str, kind: WorkloadType, name: &str, local: u16, remote: u16) -> ForwardSpec {
    ForwardSpec {
        service_id: id.into(),
        cluster: cluster(),
        namespace: NS.into(),
        kind,
        name: name.into(),
        address: "localhost".into(),
        ports: vec![PortPair { local, remote }],
    }
}

async fn http_get(port: u16, half_close: bool) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream
        .write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")
        .await?;
    if half_close {
        stream.shutdown().await?;
    }
    let mut response = String::new();
    tokio::time::timeout(
        Duration::from_secs(20),
        stream.read_to_string(&mut response),
    )
    .await
    .map_err(|_| std::io::Error::from(std::io::ErrorKind::TimedOut))??;
    Ok(response)
}

fn pod_of(response: &str) -> &str {
    response
        .lines()
        .find_map(|line| line.strip_prefix("pod="))
        .unwrap_or_default()
}

async fn next_event(events: &mut mpsc::UnboundedReceiver<ForwardEvent>) -> ForwardEvent {
    tokio::time::timeout(Duration::from_secs(15), events.next())
        .await
        .expect("an event in time")
        .expect("event stream open")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs a cluster, see the module documentation"]
async fn connection_check_and_autocomplete() {
    assert_eq!(
        k8s::check_connection(&cluster()).await.unwrap(),
        k8s::ConnectionCheck::Ok
    );

    let client = kubeconfig::client_for(&cluster()).await.unwrap();
    assert!(
        k8s::list_namespaces(&client)
            .await
            .unwrap()
            .contains(&NS.to_string())
    );
    let pods = k8s::list_workload_names(&client, NS, WorkloadType::Pod)
        .await
        .unwrap();
    assert!(pods.contains(&"standalone".to_string()), "{pods:?}");
    assert_eq!(
        k8s::list_workload_names(&client, NS, WorkloadType::Deployment)
            .await
            .unwrap(),
        ["web"]
    );
    assert_eq!(
        k8s::list_workload_names(&client, NS, WorkloadType::Service)
            .await
            .unwrap(),
        ["headless-no-selector", "web"]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs a cluster, see the module documentation"]
async fn forwards_pods_deployments_and_services() {
    let (manager, mut events) = manager();
    let [pod_port, deployment_port, service_port] = free_ports();

    let started = manager
        .start(spec("pod", WorkloadType::Pod, "standalone", pod_port, 8080))
        .await
        .unwrap();
    assert_eq!(started[0].pod, "standalone");
    assert!(started[0].addresses.iter().any(|address| address.is_ipv4()));

    let started = manager
        .start(spec(
            "deployment",
            WorkloadType::Deployment,
            "web",
            deployment_port,
            8080,
        ))
        .await
        .unwrap();
    assert!(
        started[0].pod.starts_with("web-"),
        "matchLabels + matchExpressions select the pods"
    );

    let started = manager
        .start(spec(
            "service",
            WorkloadType::Service,
            "web",
            service_port,
            80,
        ))
        .await
        .unwrap();
    assert_eq!(
        started[0].target_port, 8080,
        "the named target port is mapped"
    );

    assert_eq!(
        pod_of(&http_get(pod_port, false).await.unwrap()),
        "standalone"
    );
    assert!(pod_of(&http_get(deployment_port, false).await.unwrap()).starts_with("web-"));
    assert!(pod_of(&http_get(service_port, false).await.unwrap()).starts_with("web-"));
    // Clients that half-close their side still receive the answer.
    assert!(pod_of(&http_get(deployment_port, true).await.unwrap()).starts_with("web-"));

    // The OPTIONS probe marks all three ports as HTTP.
    let mut http_ports = HashSet::new();
    while http_ports.len() < 3 {
        match next_event(&mut events).await {
            ForwardEvent::HttpDetected { local_port, .. } => {
                http_ports.insert(local_port);
            }
            other => panic!("unexpected event {other:?}"),
        }
    }
    assert_eq!(
        http_ports,
        HashSet::from([pod_port, deployment_port, service_port])
    );

    // Concurrent connections are independent streams.
    let responses =
        futures::future::join_all((0..20).map(|_| http_get(deployment_port, false))).await;
    for response in responses {
        assert!(pod_of(&response.unwrap()).starts_with("web-"));
    }

    // A port can only be used once.
    let error = manager
        .start(spec(
            "other",
            WorkloadType::Pod,
            "standalone",
            deployment_port,
            8080,
        ))
        .await
        .unwrap_err();
    assert_eq!(
        error.message,
        format!("Port localhost:{deployment_port} is already in use.")
    );
    assert!(!manager.is_running("other"));
    assert!(manager.is_running("deployment"));

    // Stopping frees the port and it can be used again right away.
    manager.stop("deployment");
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        TcpStream::connect(("127.0.0.1", deployment_port))
            .await
            .is_err()
    );
    manager
        .start(spec(
            "deployment",
            WorkloadType::Deployment,
            "web",
            deployment_port,
            8080,
        ))
        .await
        .unwrap();
    assert!(pod_of(&http_get(deployment_port, false).await.unwrap()).starts_with("web-"));

    manager.stop_all();
    assert!(!manager.is_running("pod"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs a cluster, see the module documentation"]
async fn follows_pods_that_were_replaced() {
    let (manager, _events) = manager();
    let [port] = free_ports();
    let started = manager
        .start(spec(
            "deployment",
            WorkloadType::Deployment,
            "web",
            port,
            8080,
        ))
        .await
        .unwrap();
    let first_pod = started[0].pod.clone();
    assert_eq!(pod_of(&http_get(port, false).await.unwrap()), first_pod);

    // The pod disappears (eviction, rollout, ...). The deployment still has a running pod.
    let client = kubeconfig::client_for(&cluster()).await.unwrap();
    let pods: Api<Pod> = Api::namespaced(client, NS);
    pods.delete(
        &first_pod,
        &DeleteParams {
            grace_period_seconds: Some(0),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    for _ in 0..50 {
        if pods.get_opt(&first_pod).await.unwrap().is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let response = http_get(port, false).await.unwrap();
    let pod = pod_of(&response);
    assert!(pod.starts_with("web-") && pod != first_pod, "{response}");

    manager.stop_all();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs a cluster, see the module documentation"]
async fn reports_errors() {
    let (manager, mut events) = manager();
    let [a, b, c, port] = free_ports();

    let error = manager
        .start(spec("x", WorkloadType::Deployment, "missing", a, 80))
        .await
        .unwrap_err();
    assert_eq!(error.message, "Deployment \"missing\" not found.");

    let error = manager
        .start(spec("x", WorkloadType::Service, "web", b, 1234))
        .await
        .unwrap_err();
    assert_eq!(
        error.message,
        "Service \"web\" does not have a service port 1234. Available ports: 80"
    );

    let error = manager
        .start(spec(
            "x",
            WorkloadType::Service,
            "headless-no-selector",
            c,
            80,
        ))
        .await
        .unwrap_err();
    assert_eq!(
        error.message,
        "Service \"headless-no-selector\" does not have a selector."
    );
    assert!(!manager.is_running("x"));

    // Nothing listens on 9999 inside the pod: the local port keeps listening and the error
    // of the cluster is reported.
    manager
        .start(spec("closed", WorkloadType::Pod, "standalone", port, 9999))
        .await
        .unwrap();
    let response = http_get(port, false).await.unwrap_or_default();
    assert!(!response.starts_with("HTTP/"));
    match next_event(&mut events).await {
        ForwardEvent::ConnectionFailed {
            local_port, error, ..
        } => {
            assert_eq!(local_port, port);
            assert!(error.message.contains("9999"), "{error:?}");
            assert!(error.details.is_some());
        }
        other => panic!("unexpected event {other:?}"),
    }
    assert!(manager.is_running("closed"));
    manager.stop_all();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs a cluster, see the module documentation"]
async fn a_start_can_be_cancelled() {
    let (manager, _events) = manager();
    let [port] = free_ports();
    let start = manager.start(spec("slow", WorkloadType::Deployment, "web", port, 8080));
    manager.stop("slow");
    let error = start.await.unwrap_err();
    assert!(error.is_cancelled());
    assert!(!manager.is_running("slow"));
    assert!(TcpStream::connect(("127.0.0.1", port)).await.is_err());
}
