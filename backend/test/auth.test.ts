import { before, test } from 'node:test'
import assert from 'node:assert/strict'
import request from 'supertest'
import type { Application } from 'express'
import { bootApp } from './helpers/app'

let app: Application

before(async () => {
  app = await bootApp()
})

test('protected route rejects a missing token', async () => {
  const res = await request(app).get('/auth/me')

  assert.equal(res.status, 401)
  assert.equal(res.body.code, 'UNAUTHORIZED')
})

test('auth register reports EVERY invalid field at once', async () => {
  const res = await request(app)
    .post('/auth/register')
    .send({ name: 'a', email: 'not-an-email', password: 'short' })

  assert.equal(res.status, 400)
  assert.equal(res.body.code, 'VALIDATION_ERROR')
  assert.equal(res.body.success, false)

  const errors = res.body.data?.errors as Array<{field: string; message: string; value: unknown}> | undefined
  const fields = (errors ?? []).map((entry) => entry.field)
  assert.ok(fields.includes('body.name'))
  assert.ok(fields.includes('body.email'))
  assert.ok(fields.includes('body.password'))
  assert.ok(res.body.data?.errors?.length >= 3)

  const nameIssue = errors?.find((entry) => entry.field === 'body.name')
  assert.equal(typeof nameIssue?.message, 'string')
  assert.equal(nameIssue?.value, 'a')
})

test('valid registration reaches the service/database layer', async () => {
  // DB is disabled, so the flow must fail with a mapped DATABASE_ERROR and
  // never leak a driver error to the client.
  const res = await request(app)
    .post('/auth/register')
    .send({ name: 'John Doe', email: 'john@example.com', password: 'StrongPass1!' })

  assert.equal(res.status, 500)
  assert.equal(res.body.code, 'DATABASE_ERROR')
  assert.equal(res.body.success, false)
  assert.equal(res.body.data, null)
})

test('users admin endpoint rejects unauthenticated access', async () => {
  const res = await request(app).get('/users')
  assert.equal(res.status, 401)
  assert.equal(res.body.code, 'UNAUTHORIZED')
})

test('users admin endpoint rejects non-admin role with 403 Forbidden', async () => {
  const jwtService = (await import('../src/services/jwtService')).default
  const regularUserToken = jwtService.generateAccessToken({
    userId: 10,
    email: 'user@example.com',
    role: 'user'
  })

  const res = await request(app)
    .get('/users')
    .set('Authorization', `Bearer ${regularUserToken}`)

  assert.equal(res.status, 403)
  assert.equal(res.body.code, 'FORBIDDEN')
})

test('refresh endpoint rejects request when neither cookie nor body provides refreshToken', async () => {
  const res = await request(app).post('/auth/refresh').send({})

  assert.equal(res.status, 400)
  assert.equal(res.body.code, 'REFRESH_TOKEN_REQUIRED')
})