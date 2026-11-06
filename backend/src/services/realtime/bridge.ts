export type RealtimeHandler = (channel: string, payload: string) => void

/**
 * Realtime bridge abstraction.
 *
 * SSE and WebSocket services should NOT talk to a specific broker directly.
 * Instead they publish/subscribe through this interface, which is implemented
 * over Redis pub/sub or RabbitMQ topic exchanges. That makes every realtime
 * channel horizontally scalable across multiple app instances.
 */
export interface RealtimeBridge {
  init(): Promise<void>
  publish(channel: string, payload: string): Promise<void>
  subscribe(pattern: string, handler: RealtimeHandler): Promise<void>
  close(): Promise<void>
}

/**
 * Convert a glob pattern like `sse:*` into a regular expression.
 * The only supported wildcard is `*`.
 */
export function patternToRegExp (pattern: string): RegExp {
  const escaped = pattern.replace(/[.+^${}()[\]\\]/g, '\\$&')
  const globbed = escaped.replace(/\*/g, '.*')
  return new RegExp(`^${globbed}$`)
}