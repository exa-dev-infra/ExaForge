import RedisClient from '../../config/redis'
import { features } from '../../config'
import type { RealtimeBridge, RealtimeHandler } from './bridge'

/**
 * Redis pub/sub implementation of RealtimeBridge.
 * Uses the shared Redis client for publishing and a dedicated subscriber
 * client for consuming cross-instance messages.
 */
export class RedisBridge implements RealtimeBridge {
  private publisher: RedisClient
  private subscriber: any = null

  constructor () {
    this.publisher = RedisClient.getInstance()
  }

  async init (): Promise<void> {
    if (!features.redis) {
      throw new Error('Redis is disabled but REALTIME_BRIDGE is set to "redis"')
    }

    if (!this.subscriber) {
      this.subscriber = this.publisher.getClient().duplicate()
      await this.subscriber.connect()
    }
  }

  async publish (channel: string, payload: string): Promise<void> {
    await this.publisher.getClient().publish(channel, payload)
  }

  async subscribe (pattern: string, handler: RealtimeHandler): Promise<void> {
    await this.subscriber.pSubscribe(pattern, (message: string, channel: string) => {
      handler(channel, message)
    })
  }

  async close (): Promise<void> {
    if (this.subscriber) {
      try {
        if (this.subscriber.isOpen) {
          await this.subscriber.disconnect()
        }
      } finally {
        this.subscriber = null
      }
    }
  }
}