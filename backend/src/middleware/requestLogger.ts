import type { Request, Response } from 'express'
import logger from '../utils/logger'

const EXCLUDED_PATHS = ['/health', '/favicon.ico', '/public']

/**
 * Request logging middleware.
 * Emits one JSON log line per request (on 'finish') tagged with the request id,
 * method, url, http status and duration.
 */
export function requestLogger (req: Request, res: Response, next: () => void): void {
  if (EXCLUDED_PATHS.includes(req.path)) {
    next()
    return
  }

  const started = Date.now()

  res.on('finish', () => {
    const durationMs = Date.now() - started
    const status = res.statusCode
    const logData = {
      requestId: req.requestId,
      method: req.method,
      url: req.originalUrl,
      status,
      durationMs
    }

    if (status >= 500) {
      logger.error('request completed', logData)
    } else if (status >= 400) {
      logger.warn('request completed', logData)
    } else {
      logger.info('request completed', logData)
    }
  })

  next()
}