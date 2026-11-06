import { features } from '../../config'
import type { RealtimeBridge } from './bridge'
import { RedisBridge } from './redisBridge'
import { RabbitMQBridge } from './rabbitmqBridge'

/**
 * Factory that returns the realtime bridge configured through
 * `REALTIME_BRIDGE=redis|rabbitmq`.
 *
 * Each service that needs a subscription loop should call {@link createBridge}
 * and keep its own instance, so closing one service does not affect another.
 */
export function createBridge (): RealtimeBridge {
  return features.realtimeBridge === 'rabbitmq'
    ? new RabbitMQBridge()
    : new RedisBridge()
}

export { RealtimeBridge, patternToRegExp, RealtimeHandler } from './bridge'