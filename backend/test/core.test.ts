import { before, test } from 'node:test'
import assert from 'node:assert/strict'
import request from 'supertest'
import type { Application } from 'express'
import { bootApp } from './helpers/app'

let app: Application

before(async () => {
  app = await bootApp()
})

test('response envelope + health endpoint', async () => {
  const res = await request(app).get('/health')

  assert.equal(res.status, 200)
  assert.equal(res.body.success, true)
  assert.equal(res.body.code, 'SUCCESS')
  assert.equal(typeof res.body.message, 'string')
  assert.equal(typeof res.body.timestamp, 'string')
  assert.equal(typeof res.body.requestId, 'string')
})

test('unknown routes produce a NOT_FOUND envelope', async () => {
  const res = await request(app).get('/no-such-route')

  assert.equal(res.status, 404)
  assert.equal(res.body.success, false)
  assert.equal(res.body.code, 'NOT_FOUND')
  assert.equal(res.body.data, null)
})