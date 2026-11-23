#!/usr/bin/env bash
# Runs the end-to-end tests against a disposable k3d cluster. ~/.kube/config is not touched.
#
#   scripts/e2e.sh                  # create the cluster, run the tests, delete the cluster
#   KEEP_CLUSTER=1 scripts/e2e.sh   # keep the cluster afterwards (k3d cluster delete kf-e2e)
#   E2E_UI=0 scripts/e2e.sh         # skip the UI test (for example on Linux CI)
set -euo pipefail

cd "$(dirname "$0")/.."
CLUSTER="${CLUSTER:-kf-e2e}"
WORK="$(mktemp -d)"
export KUBECONFIG="$WORK/kubeconfig"

cleanup() {
  if [[ -z "${KEEP_CLUSTER:-}" ]]; then
    k3d cluster delete "$CLUSTER" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

if ! k3d cluster get "$CLUSTER" >/dev/null 2>&1; then
  k3d cluster create "$CLUSTER" \
    --kubeconfig-update-default=false \
    --kubeconfig-switch-context=false \
    --no-lb \
    --k3s-arg "--disable=traefik@server:0" \
    --k3s-arg "--disable=metrics-server@server:0" \
    --wait --timeout 240s
fi
k3d kubeconfig get "$CLUSTER" > "$KUBECONFIG"

kubectl apply -f crates/kf-core/tests/fixtures/e2e.yaml
kubectl -n kf-e2e rollout status deploy/web --timeout=180s
kubectl -n kf-e2e wait pod/standalone --for=condition=Ready --timeout=180s

KF_E2E_KUBECONFIG="$KUBECONFIG" cargo test -p kf-core --test e2e -- --ignored --test-threads=1
if [[ "${E2E_UI:-1}" != "0" ]]; then
  KF_E2E_KUBECONFIG="$KUBECONFIG" cargo test -p kube-forwarder -- --ignored --test-threads=1
fi
