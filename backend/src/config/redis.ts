import type { RedisClientType } from 'redis'
import { createClient } from 'redis'
import dotenv from 'dotenv'
import logger from '../utils/logger'

dotenv.config()

class RedisClient {
  private client: RedisClientType
  private static instance: RedisClient
  private isConnected: boolean = false

  private constructor () {
    const redisConfig = process.env.REDIS_URL
      ? { url: process.env.REDIS_URL }
      : {
        socket: {
          host: process.env.REDIS_HOST || 'localhost',
          port: parseInt(process.env.REDIS_PORT || '6379')
        },
        username: process.env.REDIS_USERNAME || undefined,
        password: process.env.REDIS_PASSWORD || undefined,
        database: parseInt(process.env.REDIS_DB || '0')
      }

    this.client = createClient(redisConfig)

    // Event handlers
    this.client.on('error', (err) => {
      logger.error('Redis Client Error:', { err })
    })

    this.client.on('connect', () => {
      logger.info('Redis client connected')
      this.isConnected = true
    })

    this.client.on('disconnect', () => {
      logger.info('Redis client disconnected')
      this.isConnected = false
    })
  }

  public static getInstance (): RedisClient {
    if (!RedisClient.instance) {
      RedisClient.instance = new RedisClient()
    }
    return RedisClient.instance
  }

  public async connect (): Promise<void> {
    if (!this.isConnected) {
      await this.client.connect()
    }
  }

  public async disconnect (): Promise<void> {
    if (this.isConnected) {
      await this.client.disconnect()
    }
  }

  public getClient (): RedisClientType {
    return this.client
  }

  // Common Redis operations
  public async get (key: string): Promise<string | null> {
    return await this.client.get(key)
  }

  public async set (key: string, value: string, expirySeconds?: number): Promise<void> {
    if (expirySeconds) {
      await this.client.setEx(key, expirySeconds, value)
    } else {
      await this.client.set(key, value)
    }
  }

  public async del (key: string): Promise<number> {
    return await this.client.del(key)
  }

  public async exists (key: string): Promise<number> {
    return await this.client.exists(key)
  }

  public async expire (key: string, seconds: number): Promise<boolean> {
    return await this.client.expire(key, seconds)
  }

  public async ttl (key: string): Promise<number> {
    return await this.client.ttl(key)
  }

  public async hSet (key: string, field: string, value: string): Promise<number> {
    return await this.client.hSet(key, field, value)
  }

  public async hGet (key: string, field: string): Promise<string | undefined> {
    return await this.client.hGet(key, field)
  }

  public async hGetAll (key: string): Promise<Record<string, string>> {
    return await this.client.hGetAll(key)
  }

  public async hDel (key: string, field: string): Promise<number> {
    return await this.client.hDel(key, field)
  }
}

export default RedisClient
