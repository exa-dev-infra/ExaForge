import type { Response } from 'express'
import { appEnv } from '../config'

export interface AuthTokens {
  accessToken: string
  refreshToken?: string
}

const ACCESS_TOKEN_MAX_AGE_MS = 15 * 60 * 1000 // 15 minutes
const REFRESH_TOKEN_MAX_AGE_MS = 7 * 24 * 60 * 60 * 1000 // 7 days

/**
 * Attach standardized authentication cookies to the response:
 * - `access_token`: client-readable cookie (`httpOnly: false`) so SPA frontend can inspect/use it.
 * - `refresh_token`: strictly protected cookie (`httpOnly: true`) immune to frontend XSS.
 */
export const setAuthCookies = (res: Response, tokens: AuthTokens): void => {
  const isSecure = appEnv.isProduction
  const sameSite = isSecure ? 'strict' : 'lax'

  // Access token cookie (accessible by FE javascript)
  res.cookie('access_token', tokens.accessToken, {
    httpOnly: false,
    secure: isSecure,
    sameSite,
    maxAge: ACCESS_TOKEN_MAX_AGE_MS,
    path: '/'
  })

  // Refresh token cookie (httpOnly)
  if (tokens.refreshToken) {
    res.cookie('refresh_token', tokens.refreshToken, {
      httpOnly: true,
      secure: isSecure,
      sameSite,
      maxAge: REFRESH_TOKEN_MAX_AGE_MS,
      path: '/'
    })
  }
}

/**
 * Clear authentication cookies on logout or session invalidation.
 */
export const clearAuthCookies = (res: Response): void => {
  const isSecure = appEnv.isProduction
  const sameSite = isSecure ? 'strict' : 'lax'

  res.clearCookie('access_token', {
    httpOnly: false,
    secure: isSecure,
    sameSite,
    path: '/'
  })
  res.clearCookie('refresh_token', {
    httpOnly: true,
    secure: isSecure,
    sameSite,
    path: '/'
  })
}
