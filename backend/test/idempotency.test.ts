import { test } from 'node:test'
import assert from 'node:assert/strict'
import express from 'express'
import request from 'supertest'
import {
  idempotencyMiddleware,
  MemoryIdempotencyStorage
} from '../src/middleware/idempotency'
import ErrorHandler from '../src/middleware/errorHandler'
import { bootApp } from './helpers/app'

test('idempotency feature is reported in /health', async () => {
  const app = await bootApp()
  const res = await request(app).get('/health')

  assert.equal(res.status, 200)
  assert.equal(res.body.success, true)
  assert.equal(res.body.data.features.idempotency, true)
})

test('idempotency middleware full lifecycle (replay, mismatch, conflict, retry)', async () => {
  const storage = new MemoryIdempotencyStorage()
  let executionCount = 0

  const app = express()
  app.use(express.json())

  app.use(
    idempotencyMiddleware({
      storage,
      lifetimeSeconds: 300,
      lockTimeoutSeconds: 30,
      enforceForMethods: ['POST'],
      userIdentifier: () => 'test-user-1'
    })
  )

  app.post('/test-orders', (req, res, next) => {
    executionCount++
    if (req.body.fail) {
      const err = new Error('Database connection lost')
      return next(err)
    }
    res.status(201).json({ orderId: 'ord-123', count: executionCount, item: req.body.item })
  })

  app.get('/test-orders', (req, res) => {
    res.status(200).json({ ok: true })
  })

  app.use(ErrorHandler.middleware)

  // 1. GET requests ignore Idempotency-Key
  const getRes = await request(app)
    .get('/test-orders')
    .set('Idempotency-Key', 'key-get')

  assert.equal(getRes.status, 200)
  assert.equal(getRes.headers['idempotent-replayed'], undefined)

  // 2. First POST request with Idempotency-Key executes handler
  const post1 = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'order-key-100')
    .send({ item: 'laptop' })

  assert.equal(post1.status, 201)
  assert.equal(post1.body.orderId, 'ord-123')
  assert.equal(post1.body.count, 1)
  assert.equal(post1.headers['idempotency-key'], 'order-key-100')
  assert.equal(post1.headers['idempotent-replayed'], undefined)
  assert.equal(executionCount, 1)

  // 3. Second identical POST request replays cached response without executing handler
  const post2 = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'order-key-100')
    .send({ item: 'laptop' })

  assert.equal(post2.status, 201)
  assert.equal(post2.body.orderId, 'ord-123')
  assert.equal(post2.body.count, 1)
  assert.equal(post2.headers['idempotent-replayed'], 'true')
  assert.equal(post2.headers['x-idempotency-replayed'], 'true')
  assert.equal(executionCount, 1, 'handler must NOT be called on replay')

  // 4. Same key but different payload returns 422 Unprocessable Entity
  const postMismatch = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'order-key-100')
    .send({ item: 'different-item' })

  assert.equal(postMismatch.status, 422)
  assert.equal(postMismatch.body.code, 'IDEMPOTENCY_PAYLOAD_MISMATCH')

  // 5. In-flight concurrent request returns 409 Conflict
  await storage.lock('test-user-1:in-flight-key', 'dummy-hash', 30)

  const postConflict = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'in-flight-key')
    .send({ item: 'phone' })

  assert.equal(postConflict.status, 409)
  assert.equal(postConflict.body.code, 'IDEMPOTENCY_CONFLICT')

  // 6. When request fails with 500, lock is cleared to allow retry
  const postFail = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'retry-key-500')
    .send({ item: 'error-item', fail: true })

  assert.equal(postFail.status, 500)

  // Retry with same key succeeds because lock was released
  const postRetry = await request(app)
    .post('/test-orders')
    .set('Idempotency-Key', 'retry-key-500')
    .send({ item: 'success-item' })

  assert.equal(postRetry.status, 201)
  assert.equal(postRetry.body.item, 'success-item')
})
