import { initializeTracing } from './config/tracing'

// Initialize tracing before anything else
initializeTracing()

import type { Application } from 'express'
import express from 'express'
import cookieParser from 'cookie-parser'
import dotenv from 'dotenv'
import http from 'http'

import { appEnv, features, idempotencyConfig } from './config'
import Database from './config/database'
import RedisClient from './config/redis'
import RabbitMQClient from './config/rabbitmq'

import SSEService from './services/sseService'
import WebSocketService from './services/websocketService'

import ErrorHandler from './middleware/errorHandler'
import { requestIdMiddleware } from './middleware/requestId'
import { requestLogger } from './middleware/requestLogger'
import {
  idempotencyMiddleware,
  RedisIdempotencyStorage,
  MemoryIdempotencyStorage
} from './middleware/idempotency'

import logger from './utils/logger'

import { mountModules } from './modules'

dotenv.config()

import { ApiResponse } from './types/response'

const app: Application = express()
const httpServer = http.createServer(app)

// Global middleware (order matters)
app.use(requestIdMiddleware)
app.use(requestLogger)
app.use(express.json())
app.use(express.urlencoded({ extended: false }))
app.use(cookieParser())
app.use(express.static('public'))

// Idempotency middleware for duplicate mutation requests
if (features.idempotency) {
  const storage = features.redis
    ? new RedisIdempotencyStorage(RedisClient.getInstance())
    : new MemoryIdempotencyStorage()

  app.use(
    idempotencyMiddleware({
      storage,
      lifetimeSeconds: idempotencyConfig.lifetimeHours * 3600,
      lockTimeoutSeconds: idempotencyConfig.lockTimeoutSeconds,
      header: idempotencyConfig.header
    })
  )
}

// Basic Endpoints
app.get('/', (req, res) => {
  res.status(200).json(
    ApiResponse.success(
      {
        app: 'TypeScript API Template',
        status: 'running',
        time: new Date().toISOString()
      },
      'Welcome to TypeScript Express API Template'
    )
  )
})

app.get('/health', async (req, res) => {
  const health: Record<string, any> = {
    status: 'healthy',
    app: 'TypeScript API Template',
    features: {
      redis: features.redis,
      rabbitmq: features.rabbitmq,
      database: features.database,
      idempotency: features.idempotency,
      sse: features.sse,
      websocket: features.websocket,
      realtime_bridge: features.realtimeBridge
    },
    services: {}
  }

  if (features.database) {
    try {
      const db = Database.getInstance()
      await db.query('SELECT 1')
      health.services.database = 'connected'
    } catch (e) {
      health.services.database = 'error'
      health.status = 'degraded'
    }
  } else {
    health.services.database = 'disabled'
  }

  if (features.redis) {
    const redis = RedisClient.getInstance()
    if (redis.getClient() && redis.getClient().isReady) {
      health.services.redis = 'connected'
    } else {
      health.services.redis = 'disconnected'
      health.status = 'degraded'
    }
  } else {
    health.services.redis = 'disabled'
  }

  if (features.rabbitmq) {
    try {
      const rabbitmq = RabbitMQClient.getInstance()
      const channel = rabbitmq.getChannel()
      if (channel) {
        health.services.rabbitmq = 'connected'
      } else {
        health.services.rabbitmq = 'disconnected'
        health.status = 'degraded'
      }
    } catch (e) {
      health.services.rabbitmq = 'error'
      health.status = 'degraded'
    }
  } else {
    health.services.rabbitmq = 'disabled'
  }

  res.status(200).json(ApiResponse.success(health, 'Health check completed'))
})

// Feature modules
mountModules(app)

// 404 handler - must be after all routes
app.use(ErrorHandler.notFound)

// Global error handler - must be last
app.use(ErrorHandler.middleware)

const initializeConnections = async (): Promise<void> => {
  try {
    if (features.database) {
      const db = Database.getInstance()
      await db.query('SELECT NOW()')
      logger.info('PostgreSQL connected')
    } else {
      logger.warn('PostgreSQL disabled via DATABASE_ENABLED=false')
    }

    if (features.redis) {
      const redis = RedisClient.getInstance()
      await redis.connect()
      logger.info('Redis connected')
    } else {
      logger.warn('Redis disabled via REDIS_ENABLED=false')
    }

    if (features.rabbitmq) {
      const rabbitmq = RabbitMQClient.getInstance()
      await rabbitmq.connect()
      logger.info('RabbitMQ connected')
    } else {
      logger.warn('RabbitMQ disabled via RABBITMQ_ENABLED=false')
    }

    if (features.sse) {
      await SSEService.getInstance().initialize()
    }

    if (features.websocket) {
      await WebSocketService.getInstance().initialize(httpServer)
    }

    logger.info('All enabled services connected successfully', {
      realtimeBridge: features.realtimeBridge
    })
  } catch (error) {
    logger.error('Failed to initialize connections', { error })
    process.exit(1)
  }
}

const gracefulShutdown = async (): Promise<void> => {
  logger.info('Shutting down gracefully...')

  try {
    if (features.sse) {
      await SSEService.getInstance().cleanup()
    }

    if (features.websocket) {
      await WebSocketService.getInstance().cleanup()
    }

    if (features.database) {
      const db = Database.getInstance()
      await db.close()
    }

    if (features.redis) {
      const redis = RedisClient.getInstance()
      await redis.disconnect()
    }

    if (features.rabbitmq) {
      const rabbitmq = RabbitMQClient.getInstance()
      await rabbitmq.disconnect()
    }

    process.exit(0)
  } catch (error) {
    logger.error('Error during shutdown', { error })
    process.exit(1)
  }
}

process.on('SIGTERM', gracefulShutdown)
process.on('SIGINT', gracefulShutdown)

const startServer = async (): Promise<void> => {
  await initializeConnections()

  httpServer.listen(appEnv.port, () => {
    logger.info('Server started', {
      port: appEnv.port,
      env: appEnv.nodeEnv,
      sse: features.sse,
      websocket: features.websocket,
      realtimeBridge: features.realtimeBridge
    })
  })
}

if (process.env.NODE_ENV !== 'test') {
  startServer()
}

// When imported by the integration tests (NODE_ENV=test) the server is not
// started automatically; supertest binds the app to an ephemeral port instead.

export default app
export { startServer, httpServer }