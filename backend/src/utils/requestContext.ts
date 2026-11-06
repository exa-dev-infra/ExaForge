import { AsyncLocalStorage } from 'node:async_hooks'

export interface RequestUser {
  userId: string | number
  email: string
  role?: string
}

export interface RequestContextStore {
  requestId: string
  startTime: number
  method?: string
  url?: string
  user?: RequestUser
  isAborted?: boolean
  [key: string]: unknown
}

export const requestContext = new AsyncLocalStorage<RequestContextStore>()

/**
 * Get the current request context store, or undefined if outside a request lifecycle.
 */
export const getRequestContext = (): RequestContextStore | undefined => {
  return requestContext.getStore()
}

/**
 * Get the current request ID from the active request context.
 */
export const getRequestId = (): string | undefined => {
  return requestContext.getStore()?.requestId
}

/**
 * Get the authenticated user from the active request context.
 */
export const getCurrentUser = (): RequestUser | undefined => {
  return requestContext.getStore()?.user
}

/**
 * Set or update the authenticated user in the active request context.
 */
export const setContextUser = (user: RequestUser): void => {
  const store = requestContext.getStore()
  if (store) {
    store.user = user
  }
}

/**
 * Check whether the current HTTP request has been aborted by the client.
 */
export const isRequestAborted = (): boolean => {
  return requestContext.getStore()?.isAborted === true
}

/**
 * Run a callback function within a given RequestContextStore.
 */
export const runWithContext = <R>(store: RequestContextStore, fn: () => R): R => {
  return requestContext.run(store, fn)
}
