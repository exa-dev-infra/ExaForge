import { features } from '../../config'
import Database from '../../config/database'
import RedisClient from '../../config/redis'
import RabbitMQClient from '../../config/rabbitmq'
import logger from '../../utils/logger'

export interface ServiceHealth {
  postgres: boolean
  redis: boolean
  rabbitmq: boolean
}

export class HealthService {
  /**
   * A disabled feature is considered "healthy" (skipped).
   */
  static async checkPostgres (): Promise<boolean> {
    if (!features.database) {return true}

    try {
      const db = Database.getInstance()
      await db.query('SELECT 1')
      return true
    } catch (error) {
      logger.error('PostgreSQL health check failed', { error })
      return false
    }
  }

  static async checkRedis (): Promise<boolean> {
    if (!features.redis) {return true}

    try {
      const redis = RedisClient.getInstance()
      await redis.set('health_check', 'ok', 10)
      const result = await redis.get('health_check')
      return result === 'ok'
    } catch (error) {
      logger.error('Redis health check failed', { error })
      return false
    }
  }

  static async checkRabbitMQ (): Promise<boolean> {
    if (!features.rabbitmq) {return true}

    try {
      const rabbitmq = RabbitMQClient.getInstance()
      rabbitmq.getChannel()
      return true
    } catch (error) {
      logger.error('RabbitMQ health check failed', { error })
      return false
    }
  }

  static async checkAll (): Promise<ServiceHealth> {
    const [postgres, redis, rabbitmq] = await Promise.all([
      this.checkPostgres(),
      this.checkRedis(),
      this.checkRabbitMQ()
    ])

    return { postgres, redis, rabbitmq }
  }

  static isHealthy (health: ServiceHealth): boolean {
    return health.postgres && health.redis && health.rabbitmq
  }
}

/**
 * Demo helpers used by the core routes. They wrap the infra clients so routes
 * never touch Pool/Redis/RabbitMQ directly.
 */
export async function getDatabaseNow (): Promise<unknown> {
  const db = Database.getInstance()
  const result = await db.query('SELECT NOW() as current_time, version() as postgres_version')
  return result.rows[0]
}

export async function storeAndReadRedis (
  key: string,
  value: string,
  expiry?: number
): Promise<string | null> {
  const redis = RedisClient.getInstance()
  await redis.set(key, value, expiry)
  return await redis.get(key)
}

export async function sendToQueue (queue: string, message: string): Promise<string> {
  const rabbitmq = RabbitMQClient.getInstance()
  await rabbitmq.assertQueue(queue)
  await rabbitmq.sendToQueue(queue, message)

  logger.info('Message sent to queue', { queue })
  return message
}