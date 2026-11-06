import RedisClient from '../config/redis'
import logger from '../utils/logger'

export interface CacheOptions {
    ttl?: number; // Time to live in seconds
}

export class CacheService {
  private redis: RedisClient

  constructor () {
    this.redis = RedisClient.getInstance()
  }

  async get<T> (key: string): Promise<T | null> {
    try {
      const data = await this.redis.get(key)
      if (!data) {return null}
      return JSON.parse(data) as T
    } catch (error) {
      logger.error(`Cache get error for key ${key}:`, { error })
      return null
    }
  }

  async set<T> (key: string, value: T, options?: CacheOptions): Promise<void> {
    try {
      const data = JSON.stringify(value)
      await this.redis.set(key, data, options?.ttl)
    } catch (error) {
      logger.error(`Cache set error for key ${key}:`, { error })
    }
  }

  async del (key: string): Promise<void> {
    try {
      await this.redis.del(key)
    } catch (error) {
      logger.error(`Cache delete error for key ${key}:`, { error })
    }
  }

  async exists (key: string): Promise<boolean> {
    try {
      const result = await this.redis.exists(key)
      return result > 0
    } catch (error) {
      logger.error(`Cache exists error for key ${key}:`, { error })
      return false
    }
  }

  async invalidatePattern (pattern: string): Promise<void> {
    // Note: For production, use Redis SCAN instead of KEYS
    logger.info(`Invalidating cache pattern: ${pattern}`)
    // Implementation depends on Redis version and setup
  }

  // Helper method for cache-aside pattern
  async getOrSet<T> (
    key: string,
    fetchFn: () => Promise<T>,
    options?: CacheOptions
  ): Promise<T> {
    // Try to get from cache
    const cached = await this.get<T>(key)
    if (cached !== null) {
      return cached
    }

    // Fetch from source
    const value = await fetchFn()

    // Store in cache
    await this.set(key, value, options)

    return value
  }
}

export default new CacheService()
