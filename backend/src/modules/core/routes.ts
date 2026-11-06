import type { Request, Response } from 'express'
import { Router } from 'express'
import { appEnv, features } from '../../config'
import { ApiResponse } from '../../types/response'
import ErrorHandler from '../../middleware/errorHandler'
import { validate } from '../../utils/validator'
import {
  HealthService,
  getDatabaseNow,
  sendToQueue,
  storeAndReadRedis
} from './service'
import { rabbitmqSendSchema, redisExampleSchema } from './validation'

const router = Router()

const SERVICE_STATUS: Record<string, string> = {
  disabled: 'disabled',
  connected: 'connected',
  error: 'error'
} as const

function featureStatus (enabled: boolean, connected: boolean): string {
  if (!enabled) {
    return SERVICE_STATUS.disabled
  }
  return connected ? SERVICE_STATUS.connected : SERVICE_STATUS.error
}

router.get(
  '/health',
  ErrorHandler.catchAsync(async (_req: Request, res: Response) => {
    const health = await HealthService.checkAll()
    const isHealthy = HealthService.isHealthy(health)
    const isRealtimeEnabled = features.sse || features.websocket

    const healthData = {
      status: isHealthy ? 'healthy' : 'degraded',
      app: appEnv.appName || 'typescript-template',
      features: {
        redis: features.redis,
        rabbitmq: features.rabbitmq,
        realtime: isRealtimeEnabled,
        realtime_sse: features.sse,
        realtime_ws: features.websocket,
        realtime_bridge: features.realtimeBridge
      },
      database: featureStatus(features.database, health.postgres),
      redis: featureStatus(features.redis, health.redis),
      rabbitmq: featureStatus(features.rabbitmq, health.rabbitmq),
      realtime: isRealtimeEnabled
        ? SERVICE_STATUS.connected
        : SERVICE_STATUS.disabled
    }

    const response = ApiResponse.success(healthData, 'Health check')

    res.status(isHealthy ? 200 : 503).json(response)
  })
)

router.get(
  '/db-example',
  ErrorHandler.catchAsync(async (_req: Request, res: Response) => {
    const currentTime = await getDatabaseNow()

    const response = ApiResponse.success(
      currentTime,
      'Database query executed successfully'
    )
    res.json(response)
  })
)

router.post(
  '/redis-example',
  validate(redisExampleSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const { key, value, expiry } = req.validated?.body ?? {}

    const retrieved = await storeAndReadRedis(
      String(key),
      String(value),
      Number(expiry)
    )

    const response = ApiResponse.success(
      { key, value: retrieved },
      'Value stored in Redis successfully'
    )
    res.json(response)
  })
)

router.post(
  '/rabbitmq-send',
  validate(rabbitmqSendSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const { queue, message } = req.validated?.body ?? {}

    const sent = await sendToQueue(String(queue), String(message))

    const response = ApiResponse.success(
      sent,
      `Message sent to queue: ${queue}`
    )
    res.json(response)
  })
)

router.get('/', (_req: Request, res: Response) => {
  const data = {
    message: 'Welcome to TypeScript API with PostgreSQL, Redis, and RabbitMQ',
    version: '1.0.0',
    environment: appEnv.nodeEnv,
    features,
    endpoints: {
      health: 'GET /health',
      database: 'GET /db-example',
      redis: 'POST /redis-example',
      rabbitmq: 'POST /rabbitmq-send',
      users: 'GET /users',
      auth: 'POST /auth/login',
      sse: 'GET /sse/stream',
      realtime: 'GET /realtime/stats'
    }
  }

  const response = ApiResponse.success(data, 'API is running')
  res.json(response)
})

export default router
