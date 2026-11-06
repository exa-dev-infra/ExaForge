import type { Application } from 'express'

/**
 * Shared test bootstrapper. Sets NODE_ENV=test and turns every external
 * service off before the app module is evaluated (which also stops app.ts
 * from auto-listening), then returns the fully wired Express app for supertest.
 */
let appInstance: Application | undefined

export const bootApp = async (): Promise<Application> => {
  if (appInstance) {return appInstance}

  process.env.NODE_ENV = 'test'
  process.env.DATABASE_ENABLED = 'false'
  process.env.REDIS_ENABLED = 'false'
  process.env.RABBITMQ_ENABLED = 'false'
  process.env.SSE_ENABLED = 'false'
  process.env.WS_ENABLED = 'false'
  process.env.LOG_TO_FILE = 'false'

  const imported = await import('../../src/app')
  appInstance = imported.default as Application
  return appInstance
}