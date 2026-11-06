import { before, after, test } from 'node:test'
import assert from 'node:assert/strict'
import request from 'supertest'
import type { Application } from 'express'
import { PostgreSqlContainer } from '@testcontainers/postgresql'
import { RedisContainer } from '@testcontainers/redis'
import { RabbitMQContainer } from '@testcontainers/rabbitmq'

let app: Application
let pgContainer: any
let redisContainer: any
let rmqContainer: any
before(async () => {
  process.env.NODE_ENV = 'test'
  process.env.LOG_TO_FILE = 'false'
  process.env.DATABASE_ENABLED = 'true'
  process.env.REDIS_ENABLED = 'true'
  process.env.RABBITMQ_ENABLED = 'true'
  process.env.SSE_ENABLED = 'false'
  process.env.WS_ENABLED = 'false'

  pgContainer = await new PostgreSqlContainer('postgres:15-alpine')
    .withDatabase('ts_test')
    .withUsername('testuser')
    .withPassword('testpass')
    .start()
    
  redisContainer = await new RedisContainer('redis:7-alpine').start()
  
  rmqContainer = await new RabbitMQContainer('rabbitmq:3-management-alpine')
    .withExposedPorts(5672)
    .start()

  process.env.DATABASE_URL = pgContainer.getConnectionUri()
  process.env.REDIS_URL = redisContainer.getConnectionUrl()
  
  const rmqHost = rmqContainer.getHost()
  const rmqPort = rmqContainer.getMappedPort(5672)
  process.env.RABBITMQ_URL = `amqp://guest:guest@${rmqHost}:${rmqPort}`

  // Run migrations programmatically
  const runner = (await import('node-pg-migrate')).default
  await runner({
    databaseUrl: process.env.DATABASE_URL as string,
    dir: 'migrations',
    direction: 'up',
    migrationsTable: 'pgmigrations',
    ignorePattern: '.*\\.map$'
  })

  // We must import the app dynamically AFTER setting the env variables
  const imported = await import('../src/app')
  app = imported.default as Application

  // Connect services manually
  const RedisClient = (await import('../src/config/redis')).default
  const RabbitMQClient = (await import('../src/config/rabbitmq')).default

  const redis = RedisClient.getInstance()
  await redis.connect()

  const rabbitmq = RabbitMQClient.getInstance()
  await rabbitmq.connect()
})

after(async () => {
  // Disconnect services before stopping containers
  try {
    const RedisClient = (await import('../src/config/redis')).default
    const redis = RedisClient.getInstance()
    await redis.disconnect()
  } catch (_e) {
    // ignore
  }

  try {
    const RabbitMQClient = (await import('../src/config/rabbitmq')).default
    const rabbitmq = RabbitMQClient.getInstance()
    await rabbitmq.disconnect()
  } catch (_e) {
    // ignore
  }

  try {
    const Database = (await import('../src/config/database')).default
    const db = Database.getInstance()
    await db.close()
  } catch (_e) {
    // ignore
  }

  if (pgContainer) {await pgContainer.stop()}
  if (redisContainer) {await redisContainer.stop()}
  if (rmqContainer) {await rmqContainer.stop()}
})

test('Health check shows all services connected', async () => {
  const res = await request(app).get('/health')
  
  assert.equal(res.status, 200)
  assert.equal(res.body.code, 'SUCCESS')
  assert.equal(res.body.data.status, 'healthy')
  assert.equal(res.body.data.features.redis, true)
  assert.equal(res.body.data.features.rabbitmq, true)
})

test('Auth registration and login flow end-to-end', async () => {
  const email = `integration_${Date.now()}@example.com`
  const password = 'Password123!'
  
  // 1. Register
  const regRes = await request(app)
    .post('/auth/register')
    .send({ name: 'Integration User', email, password })
    
  assert.equal(regRes.status, 201, `Register failed: ${JSON.stringify(regRes.body)}`)
  assert.equal(regRes.body.code, 'CREATED')
  assert.equal(regRes.body.data.user.email, email)
  
  // 2. Login
  const loginRes = await request(app)
    .post('/auth/login')
    .send({ email, password })
    
  assert.equal(loginRes.status, 200, `Login failed: ${JSON.stringify(loginRes.body)}`)
  assert.equal(loginRes.body.code, 'SUCCESS')
  assert.ok(loginRes.body.data.tokens.accessToken, 'Missing accessToken in login response')
  
  const token = loginRes.body.data.tokens.accessToken
  
  // 3. Get Profile
  const profileRes = await request(app)
    .get('/auth/me')
    .set('Authorization', `Bearer ${token}`)
    
  assert.equal(profileRes.status, 200, `Profile failed: ${JSON.stringify(profileRes.body)}`)
  assert.equal(profileRes.body.code, 'SUCCESS')
  assert.equal(profileRes.body.data.email, email)
})
