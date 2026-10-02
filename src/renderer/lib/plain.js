// Strip Vue 3 reactive Proxies to a plain object before crossing the
// contextBridge/IPC boundary. A Proxy isn't structured-cloneable, so passing
// a store-derived object straight to window.api throws "An object could not be
// cloned". JSON round-trip is a deep, dependency-free strip.
export const plain = value => (value == null ? value : JSON.parse(JSON.stringify(value)))
