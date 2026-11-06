import type { Application, Router } from 'express'
import { features } from '../config'

import coreRoutes from './core/routes'
import authRoutes from './auth/routes'
import usersRoutes from './users/routes'
import uploadsRoutes from './uploads/routes'
import sseRoutes from './sse/routes'
import realtimeRoutes from './realtime/routes'

/**
 * Feature module registry.
 *
 * A module is a self-contained folder under `src/modules/<name>/` exporting a
 * default Express router. Adding a feature = create the folder, then register
 * it here with its mount path. Modules gated by a feature flag use `enabled`.
 */
export interface FeatureModule {
  name: string
  mountPath: string
  enabled: boolean
  router: Router
}

export const featureModules: FeatureModule[] = [
  { name: 'core', mountPath: '/', enabled: true, router: coreRoutes },
  { name: 'auth', mountPath: '/auth', enabled: true, router: authRoutes },
  { name: 'users', mountPath: '/users', enabled: true, router: usersRoutes },
  { name: 'uploads', mountPath: '/uploads', enabled: true, router: uploadsRoutes },
  { name: 'sse', mountPath: '/sse', enabled: features.sse, router: sseRoutes },
  { name: 'realtime', mountPath: '/realtime', enabled: features.sse || features.websocket, router: realtimeRoutes }
]

/**
 * Mount all enabled feature modules onto the Express app.
 */
export const mountModules = (app: Application): void => {
  for (const module of featureModules) {
    if (module.enabled) {
      app.use(module.mountPath, module.router)
    }
  }
}