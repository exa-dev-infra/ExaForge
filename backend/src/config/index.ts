// Central application configuration.
// Every external integration (database, redis, rabbitmq, sse, websocket)
// can be turned on/off at runtime through a *_ENABLED env flag without
// touching any code.

export type RealtimeBridgeType = 'redis' | 'rabbitmq'

export interface FeaturesConfig {
  database: boolean
  redis: boolean
  rabbitmq: boolean
  sse: boolean
  websocket: boolean
  realtimeBridge: RealtimeBridgeType
  idempotency: boolean
}

export interface IdempotencyConfig {
  lifetimeHours: number
  lockTimeoutSeconds: number
  header: string
}

export interface AppEnv {
  appName: string
  nodeEnv: string
  isDevelopment: boolean
  isProduction: boolean
  port: number
  logLevel: string
  logDir: string
  logToFile: boolean
}

function parseBool (value: string | undefined, fallback: boolean): boolean {
  if (value === undefined || value === '') {return fallback}
  return ['1', 'true', 'yes', 'on'].includes(value.toLowerCase())
}

export const appEnv: AppEnv = {
  appName: process.env.APP_NAME || 'typescript-template',
  nodeEnv: process.env.NODE_ENV || 'development',
  get isDevelopment () { return this.nodeEnv === 'development' },
  get isProduction () { return this.nodeEnv === 'production' },
  port: parseInt(process.env.PORT || '3000', 10),
  logLevel: process.env.LOG_LEVEL || 'info',
  logDir: process.env.LOG_DIR || 'logs',
  logToFile: parseBool(process.env.LOG_TO_FILE, true)
}

export const idempotencyConfig: IdempotencyConfig = {
  lifetimeHours: parseInt(process.env.IDEMPOTENCY_LIFETIME_HOURS || '24', 10),
  lockTimeoutSeconds: parseInt(process.env.IDEMPOTENCY_LOCK_TIMEOUT_SECONDS || '30', 10),
  header: process.env.IDEMPOTENCY_HEADER || 'Idempotency-Key'
}

// Feature flags: `X_ENABLED=true|false` in .env
export const features: FeaturesConfig = {
  database: parseBool(process.env.DATABASE_ENABLED, true),
  redis: parseBool(process.env.REDIS_ENABLED, true),
  rabbitmq: parseBool(process.env.RABBITMQ_ENABLED, true),
  sse: parseBool(process.env.SSE_ENABLED, true),
  websocket: parseBool(process.env.WS_ENABLED, true),
  realtimeBridge: (process.env.REALTIME_BRIDGE || 'redis') as RealtimeBridgeType,
  idempotency: parseBool(process.env.IDEMPOTENCY_ENABLED, true)
}