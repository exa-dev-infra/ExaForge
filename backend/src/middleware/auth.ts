import type { NextFunction, Request, Response } from 'express'
import jwtService from '../services/jwtService'
import { ForbiddenError, UnauthorizedError } from '../types/errors'
import { setContextUser } from '../utils/requestContext'

// Extend Express Request to include user
declare global {
  namespace Express {
    interface Request {
      user?: {
        userId: string | number
        email: string
        role?: string
      }
    }
  }
}

/**
 * Authenticate middleware - verifies access token
 */
export const authenticate = (req: Request, res: Response, next: NextFunction) => {
  try {
    let token: string | undefined
    const authHeader = req.headers.authorization

    if (authHeader) {
      if (!authHeader.startsWith('Bearer ')) {
        throw new UnauthorizedError('Invalid authorization format. Use: Bearer <token>')
      }
      token = authHeader.substring(7)
    } else if (typeof req.query.token === 'string' && req.query.token.trim()) {
      token = req.query.token.trim()
    } else if (req.cookies && (typeof req.cookies.access_token === 'string' || typeof req.cookies.token === 'string')) {
      token = req.cookies.access_token || req.cookies.token
    }

    if (!token) {
      throw new UnauthorizedError('No authorization token provided')
    }

    // Verify token
    const decoded = jwtService.verifyAccessToken(token)

    // Attach user to request
    req.user = {
      userId: decoded.userId,
      email: decoded.email,
      role: decoded.role
    }
    setContextUser(req.user)

    next()
  } catch (error) {
    next(error)
  }
}

/**
 * Optional authentication - doesn't throw error if no token
 */
export const optionalAuth = (req: Request, res: Response, next: NextFunction) => {
  try {
    let token: string | undefined
    const authHeader = req.headers.authorization

    if (authHeader && authHeader.startsWith('Bearer ')) {
      token = authHeader.substring(7)
    } else if (typeof req.query.token === 'string' && req.query.token.trim()) {
      token = req.query.token.trim()
    } else if (req.cookies && (typeof req.cookies.access_token === 'string' || typeof req.cookies.token === 'string')) {
      token = req.cookies.access_token || req.cookies.token
    }

    if (token) {
      const decoded = jwtService.verifyAccessToken(token)
      req.user = {
        userId: decoded.userId,
        email: decoded.email,
        role: decoded.role
      }
      setContextUser(req.user)
    }

    next()
  } catch (error) {
    // Don't throw error, just continue without user
    next()
  }
}

/**
 * Authorize middleware - checks if user has required role
 */
export const authorize = (...allowedRoles: string[]) => {
  return (req: Request, res: Response, next: NextFunction) => {
    try {
      if (!req.user) {
        throw new UnauthorizedError('Authentication required')
      }

      if (!req.user.role) {
        throw new ForbiddenError('User role not found')
      }

      if (!allowedRoles.includes(req.user.role)) {
        throw new ForbiddenError(
          `Access denied. Required roles: ${allowedRoles.join(', ')}`
        )
      }

      next()
    } catch (error) {
      next(error)
    }
  }
}

/**
 * Check if user owns the resource
 */
export const checkOwnership = (resourceUserIdField: string = 'userId') => {
  return (req: Request, res: Response, next: NextFunction) => {
    try {
      if (!req.user) {
        throw new UnauthorizedError('Authentication required')
      }

      // Get resource user ID from params or body
      const resourceUserId =
                req.params[resourceUserIdField] ||
                req.body[resourceUserIdField] ||
                req.query[resourceUserIdField]

      if (!resourceUserId) {
        throw new ForbiddenError('Resource user ID not found')
      }

      // Check if user owns the resource or is admin
      if (
        req.user.userId.toString() !== resourceUserId.toString() &&
                req.user.role !== 'admin'
      ) {
        throw new ForbiddenError('Access denied. You can only access your own resources')
      }

      next()
    } catch (error) {
      next(error)
    }
  }
}

/**
 * Rate limiting by user (stored in Redis)
 */
export const rateLimitByUser = (_maxRequests: number = 100, _windowMs: number = 60000) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      if (!req.user) {
        throw new UnauthorizedError('Authentication required')
      }

      // This is a placeholder - implement with Redis
      // const redisClient = RedisClient.getInstance();
      // const key = `rate_limit:${req.user.userId}`;
      // ... implement rate limiting logic

      next()
    } catch (error) {
      next(error)
    }
  }
}
