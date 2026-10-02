import { describe, it, expect } from 'vitest'
import { reactive } from 'vue'
import { plain } from '../src/renderer/lib/plain'

// P1: plain() is the fix for k8s calls throwing "An object could not be cloned".
// Store objects are Vue reactive Proxies, which aren't structured-cloneable across
// the contextBridge/IPC boundary. plain() must return a proxy-free deep copy.
describe('plain', () => {
  it('strips a Vue reactive Proxy to a structured-cloneable plain object', () => {
    const cluster = reactive({ name: 'prod', config: { server: 'https://x', ports: [80, 443] } })

    const result = plain(cluster)

    // The actual failure mode: a Proxy throws when structured-cloned.
    expect(() => structuredClone(result)).not.toThrow()
    expect(result).toEqual({ name: 'prod', config: { server: 'https://x', ports: [80, 443] } })
    // Must be a real detached copy, not the reactive source.
    result.config.server = 'mutated'
    expect(cluster.config.server).toBe('https://x')
  })

  it('passes null and undefined through untouched', () => {
    expect(plain(null)).toBe(null)
    expect(plain(undefined)).toBe(undefined)
  })
})
