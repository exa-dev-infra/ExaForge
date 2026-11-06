import { before, test } from 'node:test'
import assert from 'node:assert/strict'
import request from 'supertest'
import type { Application } from 'express'
import { bootApp } from './helpers/app'

let app: Application

before(async () => {
  app = await bootApp()
})

test('multipart upload accepts valid files and metadata', async () => {
  const res = await request(app)
    .post('/uploads')
    .field('title', 'Report')
    .field('tags', 'one')
    .field('tags', 'two')
    .attach('avatar', Buffer.from('fake-png-content'), 'avatar.png')

  assert.equal(res.status, 201)
  assert.equal(res.body.code, 'CREATED')
  assert.equal(res.body.data.title, 'Report')
  assert.deepEqual(res.body.data.tags, ['one', 'two'])
  assert.equal(res.body.data.avatar.name, 'avatar.png')
  assert.equal(res.body.data.avatar.type, 'image/png')
  assert.deepEqual(res.body.data.documents, [])
})

test('multipart upload rejects an unsupported file type', async () => {
  const res = await request(app)
    .post('/uploads')
    .field('title', 'Report')
    .attach('avatar', Buffer.from('x'), 'note.txt')

  assert.equal(res.status, 400)
  assert.ok(
    (res.body.data?.errors as Array<{field: string}> | undefined)?.some((entry) => entry.field === 'files.avatar')
  )
})