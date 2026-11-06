import path from 'node:path'
import winston from 'winston'
import DailyRotateFile from 'winston-daily-rotate-file'
import { appEnv } from '../config'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LogContext {
  requestId?: string
  [key: string]: unknown
}

import { trace } from '@opentelemetry/api'
import { getRequestContext } from './requestContext'

const level = (appEnv.logLevel || 'info').toLowerCase()

const otelFormat = winston.format((info) => {
  const span = trace.getActiveSpan()
  if (span) {
    const spanContext = span.spanContext()
    info.trace_id = spanContext.traceId
    info.span_id = spanContext.spanId
  }
  return info
})()

const requestContextFormat = winston.format((info) => {
  const store = getRequestContext()
  if (store) {
    if (store.requestId && !info.requestId) {
      info.requestId = store.requestId
    }
    if (store.user?.userId && !info.userId) {
      info.userId = store.user.userId
    }
  }
  return info
})()

const logFormat = winston.format.combine(
  winston.format.timestamp(),
  otelFormat,
  requestContextFormat,
  winston.format.errors({ stack: true }),
  winston.format.json()
)

const transports: winston.transport[] = [
  new winston.transports.Console({
    format: logFormat
  })
]

if (appEnv.logToFile && appEnv.logDir) {
  const logDir = path.resolve(appEnv.logDir)
  transports.push(
    new DailyRotateFile({
      dirname: logDir,
      filename: 'app-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      zippedArchive: true,
      maxSize: '20m',
      maxFiles: '14d',
      format: logFormat
    })
  )
}

const winstonLogger = winston.createLogger({
  level: level,
  format: logFormat,
  transports
})

class Logger {
  debug (message: string, context?: LogContext): void {
    winstonLogger.debug(message, context)
  }

  info (message: string, context?: LogContext): void {
    winstonLogger.info(message, context)
  }

  warn (message: string, context?: LogContext): void {
    winstonLogger.warn(message, context)
  }

  error (message: string, context?: LogContext): void {
    winstonLogger.error(message, context)
  }
}

export default new Logger()