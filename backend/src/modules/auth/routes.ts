import type { Request, Response } from 'express'
import { Router } from 'express'
import { ApiResponse } from '../../types/response'
import ErrorHandler from '../../middleware/errorHandler'
import { authenticate } from '../../middleware/auth'
import { validate } from '../../utils/validator'
import { clearAuthCookies, setAuthCookies } from '../../utils/cookie'
import { RESPONSE_CODES, ValidationError } from '../../types/errors'
import authService from './service'
import {
  changePasswordSchema,
  loginSchema,
  logoutSchema,
  refreshSchema,
  registerSchema
} from './validation'

const router = Router()

/**
 * Register new user
 * POST /auth/register
 */
router.post(
  '/register',
  validate(registerSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const data = req.validated?.body as unknown as Parameters<typeof authService.register>[0]
    const meta = {
      userAgent: req.headers['user-agent'],
      ipAddress: req.ip
    }

    const result = await authService.register(data, meta)

    // Set cookies: access_token (client-readable) & refresh_token (httpOnly)
    setAuthCookies(res, result.tokens)

    const response = ApiResponse.created(result, 'User registered successfully')
    res.status(201).json(response)
  })
)

/**
 * Login user
 * POST /auth/login
 */
router.post(
  '/login',
  validate(loginSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const data = req.validated?.body as unknown as Parameters<typeof authService.login>[0]
    const meta = {
      userAgent: req.headers['user-agent'],
      ipAddress: req.ip
    }

    const result = await authService.login(data, meta)

    // Set cookies: access_token (client-readable) & refresh_token (httpOnly)
    setAuthCookies(res, result.tokens)

    const response = ApiResponse.success(result, 'Login successful')
    res.json(response)
  })
)

/**
 * Refresh access token
 * POST /auth/refresh
 * Reads refresh token from HTTP-only cookie first, falls back to request body (for Mobile).
 */
router.post(
  '/refresh',
  validate(refreshSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const cookieToken = typeof req.cookies?.refresh_token === 'string' ? req.cookies.refresh_token : undefined
    const bodyToken =
      typeof req.validated?.body?.refreshToken === 'string' ? req.validated.body.refreshToken : undefined
    const refreshToken = cookieToken || bodyToken

    if (!refreshToken) {
      throw new ValidationError(
        'Refresh token is required in cookie or body',
        { field: 'refreshToken' },
        RESPONSE_CODES.REFRESH_TOKEN_REQUIRED
      )
    }

    const meta = {
      userAgent: req.headers['user-agent'],
      ipAddress: req.ip
    }

    const result = await authService.refreshToken(refreshToken, meta)

    // If token was rotated, update both access & refresh cookies.
    // Otherwise update only access token cookie.
    if (result.rotated) {
      setAuthCookies(res, result.tokens)
    } else {
      setAuthCookies(res, { accessToken: result.tokens.accessToken })
    }

    const response = ApiResponse.success(result, 'Tokens refreshed successfully')
    res.json(response)
  })
)

/**
 * Logout user
 * POST /auth/logout
 * Invalidates refresh token in DB and clears auth cookies.
 */
router.post(
  '/logout',
  validate(logoutSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const cookieToken = typeof req.cookies?.refresh_token === 'string' ? req.cookies.refresh_token : undefined
    const bodyToken =
      typeof req.validated?.body?.refreshToken === 'string' ? req.validated.body.refreshToken : undefined
    const refreshToken = cookieToken || bodyToken
    const userId = req.user?.userId ?? req.validated?.body?.userId

    await authService.logout(userId ? Number(userId) : undefined, refreshToken)

    clearAuthCookies(res)

    const response = ApiResponse.success(null, 'Logout successful')
    res.json(response)
  })
)

/**
 * Change password
 * POST /auth/change-password
 * Requires authentication
 */
router.post(
  '/change-password',
  authenticate,
  validate(changePasswordSchema),
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const userId = req.user?.userId
    const { currentPassword, newPassword } = req.validated?.body ?? {}

    await authService.changePassword(Number(userId), String(currentPassword), String(newPassword))

    clearAuthCookies(res)

    const response = ApiResponse.success(null, 'Password changed successfully. Please login again.')
    res.json(response)
  })
)

/**
 * Get current user profile
 * GET /auth/me
 * Requires authentication
 */
router.get(
  '/me',
  authenticate,
  ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const user = await authService.getProfile(Number(req.user?.userId))

    const response = ApiResponse.success(user, 'User profile retrieved')
    res.json(response)
  })
)

export default router