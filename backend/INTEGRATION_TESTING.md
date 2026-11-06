# Integration Testing Guide

This project uses `testcontainers` for Node.js to provide a robust, isolated environment for integration testing. Real Docker containers for PostgreSQL, Redis, and RabbitMQ are dynamically spun up, tested against, and torn down automatically.

## Prerequisites
- Docker must be installed and running on your machine.
- Node.js 22.x+ (to support Node.js native test runner)

## Running the Tests

To run the integration tests, you execute the specific integration test file via `tsx`:

```bash
# Run integration tests
npx tsx --test test/integration.test.ts
```

Note: The standard `npm test` script runs the unit tests in `test/**/*.test.ts` where the database is deliberately mocked. Integration tests are isolated in `test/integration.test.ts`.

## How It Works
1. **Container Setup**: The `before` hook in `test/integration.test.ts` dynamically spins up PostgreSQL, Redis, and RabbitMQ containers.
2. **Migrations**: The setup hook uses the `node-pg-migrate` programmatic API to execute your TypeScript database migrations directly against the PostgreSQL test container.
3. **Environment Overrides**: Connection strings and credentials for the containers are injected into `process.env` before the main application is imported and evaluated.

## Writing New Integration Tests

When writing new integration tests:
1. Append them to `test/integration.test.ts` using the native Node.js `test` function.
2. Use `supertest` to make HTTP requests against the `app` instance.

Example:
```typescript
import { test } from 'node:test'
import assert from 'node:assert/strict'
import request from 'supertest'

test('My new endpoint works with DB', async () => {
  const res = await request(app).get('/api/v1/my-endpoint')
  
  assert.equal(res.status, 200)
  assert.equal(res.body.code, 'SUCCESS')
})
```
