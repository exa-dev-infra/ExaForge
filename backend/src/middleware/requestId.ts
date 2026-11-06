import { randomUUID } from 'node:crypto'
import type { NextFunction, Request, Response } from 'express'
import { requestContext, type RequestContextStore } from '../utils/requestContext'

declare global {
  namespace Express {
    interface Request {
      requestId?: string
    }
  }
}

const MAX_HEADER_LENGTH = 64

/**
 * Assigns a request id to every request and exposes it as `X-Request-ID`.
 *
 * Wraps downstream execution in an AsyncLocalStorage RequestContext so all
 * deep services, repositories, and loggers have seamless access to request state.
 *
 * Also wraps `res.json` so every JSON payload is enriched with the request id,
 * keeping the response shape uniform across all routes and error handlers.
 */
export function requestIdMiddleware (req: Request, res: Response, next: NextFunction): void {
  const incoming = req.headers['x-request-id']
  const candidate = typeof incoming === 'string' ? incoming.trim() : ''
  const requestId = candidate && candidate.length <= MAX_HEADER_LENGTH ? candidate : randomUUID()
  req.requestId = requestId

  res.setHeader('X-Request-ID', requestId)

  const originalJson = res.json.bind(res)
  res.json = (body: unknown): Response => {
    if (body && typeof body === 'object' && !Array.isArray(body)) {
      ;(body as Record<string, unknown>).requestId = req.requestId
    }
    return originalJson(body)
  }

  const store: RequestContextStore = {
    requestId,
    startTime: Date.now(),
    method: req.method,
    url: req.originalUrl || req.url,
    isAborted: false
  }

  req.on('close', () => {
    if (!res.writableEnded) {
      store.isAborted = true
    }
  })

  requestContext.run(store, () => {
    next()
  })
}