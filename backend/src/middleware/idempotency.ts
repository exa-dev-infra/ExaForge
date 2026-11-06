import type { Request, Response, NextFunction, RequestHandler } from 'express'
import crypto from 'crypto'
import RedisClient from '../config/redis'
import {
  IdempotencyConflictError,
  IdempotencyKeyRequiredError,
  IdempotencyPayloadMismatchError,
  BadRequestError
} from '../types/errors'
import logger from '../utils/logger'

export const HEADER_IDEMPOTENCY_KEY = 'Idempotency-Key'
export const HEADER_ALT_IDEMPOTENCY_KEY = 'X-Idempotency-Key'
export const HEADER_IDEMPOTENT_REPLAYED = 'Idempotent-Replayed'
export const HEADER_X_IDEMPOTENT_REPLAYED = 'X-Idempotency-Replayed'

export interface IdempotencyEntry {
  key: string
  status: 'in_progress' | 'completed'
  statusCode: number
  headers: Record<string, string>
  body: any
  requestHash: string
  createdAt: string
  expiresAt: string
}

export interface IdempotencyStorage {
  lock (
    key: string,
    requestHash: string,
    ttlSeconds: number
  ): Promise<{ acquired: boolean; existing?: IdempotencyEntry }>
  save (key: string, entry: IdempotencyEntry, ttlSeconds: number): Promise<void>
  unlock (key: string): Promise<void>
  get (key: string): Promise<IdempotencyEntry | null>
}

// ============================================================================
// Redis Storage Implementation
// ============================================================================

export class RedisIdempotencyStorage implements IdempotencyStorage {
  private redis: RedisClient
  private prefix: string

  constructor (redis?: RedisClient, prefix = 'idempotency:') {
    this.redis = redis || RedisClient.getInstance()
    this.prefix = prefix
  }

  async lock (
    key: string,
    requestHash: string,
    ttlSeconds: number
  ): Promise<{ acquired: boolean; existing?: IdempotencyEntry }> {
    const client = this.redis.getClient()
    const fullKey = this.prefix + key
    const now = new Date()
    const entry: IdempotencyEntry = {
      key,
      status: 'in_progress',
      statusCode: 0,
      headers: {},
      body: null,
      requestHash,
      createdAt: now.toISOString(),
      expiresAt: new Date(now.getTime() + ttlSeconds * 1000).toISOString()
    }

    const payload = JSON.stringify(entry)
    // Atomic SetNX with EX in node-redis v4
    const result = await client.set(fullKey, payload, {
      NX: true,
      EX: ttlSeconds
    })

    if (result === 'OK') {
      return { acquired: true }
    }

    const existing = await this.get(key)
    return { acquired: false, existing: existing || undefined }
  }

  async save (key: string, entry: IdempotencyEntry, ttlSeconds: number): Promise<void> {
    const client = this.redis.getClient()
    entry.status = 'completed'
    entry.expiresAt = new Date(Date.now() + ttlSeconds * 1000).toISOString()
    const payload = JSON.stringify(entry)
    await client.set(this.prefix + key, payload, { EX: ttlSeconds })
  }

  async unlock (key: string): Promise<void> {
    const client = this.redis.getClient()
    await client.del(this.prefix + key)
  }

  async get (key: string): Promise<IdempotencyEntry | null> {
    const client = this.redis.getClient()
    const data = await client.get(this.prefix + key)
    if (!data) {return null}
    try {
      return JSON.parse(data) as IdempotencyEntry
    } catch {
      return null
    }
  }
}

// ============================================================================
// Memory Storage Implementation (Fallback & Testing)
// ============================================================================

export class MemoryIdempotencyStorage implements IdempotencyStorage {
  private map = new Map<string, IdempotencyEntry>()

  async lock (
    key: string,
    requestHash: string,
    ttlSeconds: number
  ): Promise<{ acquired: boolean; existing?: IdempotencyEntry }> {
    const existing = this.map.get(key)
    if (existing) {
      if (new Date(existing.expiresAt) > new Date()) {
        return { acquired: false, existing: { ...existing } }
      }
      this.map.delete(key)
    }

    const now = new Date()
    const entry: IdempotencyEntry = {
      key,
      status: 'in_progress',
      statusCode: 0,
      headers: {},
      body: null,
      requestHash,
      createdAt: now.toISOString(),
      expiresAt: new Date(now.getTime() + ttlSeconds * 1000).toISOString()
    }
    this.map.set(key, entry)
    return { acquired: true }
  }

  async save (key: string, entry: IdempotencyEntry, ttlSeconds: number): Promise<void> {
    entry.status = 'completed'
    entry.expiresAt = new Date(Date.now() + ttlSeconds * 1000).toISOString()
    this.map.set(key, { ...entry })
  }

  async unlock (key: string): Promise<void> {
    this.map.delete(key)
  }

  async get (key: string): Promise<IdempotencyEntry | null> {
    const entry = this.map.get(key)
    if (!entry) {return null}
    if (new Date(entry.expiresAt) <= new Date()) {
      this.map.delete(key)
      return null
    }
    return { ...entry }
  }

  clear (): void {
    this.map.clear()
  }
}

// ============================================================================
// Middleware Definition
// ============================================================================

export interface IdempotencyOptions {
  storage?: IdempotencyStorage
  lifetimeSeconds?: number
  lockTimeoutSeconds?: number
  header?: string
  enforceForMethods?: string[]
  required?: boolean
  userIdentifier?: (req: Request) => string
}

export function computeRequestHash (userId: string, method: string, path: string, body: unknown): string {
  const hash = crypto.createHash('sha256')
  hash.update(userId)
  hash.update(':')
  hash.update(method.toUpperCase())
  hash.update(':')
  hash.update(path)
  hash.update(':')
  if (body) {
    if (typeof body === 'string' || Buffer.isBuffer(body)) {
      hash.update(body)
    } else {
      hash.update(JSON.stringify(body))
    }
  }
  return hash.digest('hex')
}

export function idempotencyMiddleware (options?: IdempotencyOptions): RequestHandler {
  const storage = options?.storage || new MemoryIdempotencyStorage()
  const lifetimeSeconds = options?.lifetimeSeconds || 24 * 3600
  const lockTimeoutSeconds = options?.lockTimeoutSeconds || 30
  const headerName = options?.header || HEADER_IDEMPOTENCY_KEY
  const enforceMethods = (options?.enforceForMethods || ['POST', 'PATCH']).map(m => m.toUpperCase())
  const required = options?.required ?? false
  const userIdentifier = options?.userIdentifier || ((req: Request) => {
    return (req as any).user?.id ? String((req as any).user.id) : (req.ip || 'anonymous')
  })

  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    // 1. Only enforce for target methods
    if (!enforceMethods.includes(req.method.toUpperCase())) {
      return next()
    }

    // 2. Extract header
    let key = req.header(headerName)
    if (!key && headerName === HEADER_IDEMPOTENCY_KEY) {
      key = req.header(HEADER_ALT_IDEMPOTENCY_KEY)
    }
    key = key?.trim()

    // 3. Handle missing key
    if (!key) {
      if (required) {
        return next(new IdempotencyKeyRequiredError())
      }
      return next()
    }

    // 4. Validate key format
    if (key.length === 0 || key.length > 256) {
      return next(new BadRequestError('Invalid Idempotency-Key header value'))
    }

    const userId = userIdentifier(req)
    const scopedKey = `${userId}:${key}`
    const reqPath = req.originalUrl || req.url
    const requestHash = computeRequestHash(userId, req.method, reqPath, req.body)

    try {
      const lockResult = await storage.lock(scopedKey, requestHash, lockTimeoutSeconds)

      if (!lockResult.acquired) {
        const existing = lockResult.existing || await storage.get(scopedKey)

        if (existing) {
          // If request is still in-flight, return 409 Conflict
          if (existing.status === 'in_progress') {
            return next(new IdempotencyConflictError())
          }

          // Verify payload fingerprint for completed requests
          if (existing.requestHash && existing.requestHash !== requestHash) {
            return next(new IdempotencyPayloadMismatchError())
          }

          // Replay cached response
          if (existing.status === 'completed') {
            res.setHeader(HEADER_IDEMPOTENCY_KEY, key)
            res.setHeader(HEADER_IDEMPOTENT_REPLAYED, 'true')
            res.setHeader(HEADER_X_IDEMPOTENT_REPLAYED, 'true')

            if (existing.headers) {
              for (const [hK, hV] of Object.entries(existing.headers)) {
                if (hV && !res.getHeader(hK)) {
                  res.setHeader(hK, hV)
                }
              }
            }

            const statusCode = existing.statusCode || 200
            res.status(statusCode)
            if (typeof existing.body === 'object' && existing.body !== null) {
              res.json(existing.body)
              return
            }
            res.send(existing.body)
            return
          }
        }

        return next(new IdempotencyConflictError())
      }

      res.setHeader(HEADER_IDEMPOTENCY_KEY, key)

      // Capture response
      const originalSend = res.send
      const originalJson = res.json

      let capturedBody: any = null
      let isCaptured = false

      res.send = function (...args: [body?: any]) {
        if (!isCaptured && args[0] !== undefined) {
          capturedBody = args[0]
          isCaptured = true
        }
        return originalSend.apply(this, args)
      }

      res.json = function (...args: [body?: any]) {
        if (!isCaptured && args[0] !== undefined) {
          capturedBody = args[0]
          isCaptured = true
        }
        return originalJson.apply(this, args)
      }

      res.on('finish', async () => {
        try {
          const statusCode = res.statusCode
          if (statusCode >= 500) {
            // Unlock so client can retry after internal error
            await storage.unlock(scopedKey)
            return
          }

          const headers: Record<string, string> = {}
          const contentType = res.getHeader('content-type')
          if (contentType) {
            headers['content-type'] = String(contentType)
          }

          // Parse string JSON if needed so json responses stay clean
          let finalBody = capturedBody
          if (typeof finalBody === 'string') {
            try {
              finalBody = JSON.parse(finalBody)
            } catch {
              // keep as string
            }
          }

          const entry: IdempotencyEntry = {
            key,
            status: 'completed',
            statusCode,
            headers,
            body: finalBody,
            requestHash,
            createdAt: new Date().toISOString(),
            expiresAt: new Date(Date.now() + lifetimeSeconds * 1000).toISOString()
          }

          await storage.save(scopedKey, entry, lifetimeSeconds)
        } catch (err) {
          logger.warn('Failed to save idempotency response', { error: err })
        }
      })

      res.on('close', async () => {
        if (!res.writableEnded) {
          try {
            await storage.unlock(scopedKey)
          } catch (err) {
            logger.warn('Failed to unlock idempotency key on connection close', { error: err })
          }
        }
      })

      return next()
    } catch (err) {
      logger.warn('Idempotency middleware error, bypassing', { error: err })
      return next()
    }
  }
}
