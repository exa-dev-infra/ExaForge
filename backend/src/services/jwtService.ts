import jwt from 'jsonwebtoken'
import { RESPONSE_CODES, UnauthorizedError } from '../types/errors'
import logger from '../utils/logger'

export interface TokenPayload {
    userId: string | number;
    email: string;
    role?: string;
}

export interface TokenResponse {
    accessToken: string;
    refreshToken: string;
}

class JWTService {
  private accessSecret: string
  private refreshSecret: string
  private accessExpiry: string
  private refreshExpiry: string

  constructor () {
    this.accessSecret = process.env.JWT_ACCESS_SECRET || 'default-access-secret'
    this.refreshSecret = process.env.JWT_REFRESH_SECRET || 'default-refresh-secret'
    this.accessExpiry = process.env.JWT_ACCESS_EXPIRY || '15m'
    this.refreshExpiry = process.env.JWT_REFRESH_EXPIRY || '7d'

    if (this.accessSecret === 'default-access-secret' || this.refreshSecret === 'default-refresh-secret') {
      logger.warn('Using default JWT secrets. Set JWT_ACCESS_SECRET and JWT_REFRESH_SECRET in .env')
    }
  }

  /**
     * Generate access token
     */
  generateAccessToken (payload: TokenPayload): string {
    return jwt.sign(
            payload as any,
            this.accessSecret,
            {
              expiresIn: this.accessExpiry
            } as jwt.SignOptions
    )
  }

  /**
     * Generate refresh token
     */
  generateRefreshToken (payload: TokenPayload): string {
    const refreshPayload = { userId: payload.userId, email: payload.email }
    return jwt.sign(
            refreshPayload as any,
            this.refreshSecret,
            {
              expiresIn: this.refreshExpiry
            } as jwt.SignOptions
    )
  }

  /**
     * Generate both access and refresh tokens
     */
  generateTokenPair (payload: TokenPayload): TokenResponse {
    return {
      accessToken: this.generateAccessToken(payload),
      refreshToken: this.generateRefreshToken(payload)
    }
  }

  /**
     * Verify access token
     */
  verifyAccessToken (token: string): TokenPayload {
    try {
      const decoded = jwt.verify(token, this.accessSecret) as TokenPayload
      return decoded
    } catch (error) {
      if (error instanceof jwt.TokenExpiredError) {
        throw new UnauthorizedError('Access token has expired', RESPONSE_CODES.ACCESS_TOKEN_EXPIRED)
      } else if (error instanceof jwt.JsonWebTokenError) {
        throw new UnauthorizedError('Invalid access token', RESPONSE_CODES.INVALID_ACCESS_TOKEN)
      }
      throw new UnauthorizedError('Token verification failed', RESPONSE_CODES.TOKEN_VERIFICATION_FAILED)
    }
  }

  /**
     * Verify refresh token
     */
  verifyRefreshToken (token: string): TokenPayload {
    try {
      const decoded = jwt.verify(token, this.refreshSecret) as TokenPayload
      return decoded
    } catch (error) {
      if (error instanceof jwt.TokenExpiredError) {
        throw new UnauthorizedError('Refresh token has expired', RESPONSE_CODES.REFRESH_TOKEN_EXPIRED)
      } else if (error instanceof jwt.JsonWebTokenError) {
        throw new UnauthorizedError('Invalid refresh token', RESPONSE_CODES.INVALID_REFRESH_TOKEN)
      }
      throw new UnauthorizedError('Token verification failed', RESPONSE_CODES.TOKEN_VERIFICATION_FAILED)
    }
  }

  /**
     * Decode token without verification (use with caution)
     */
  decodeToken (token: string): TokenPayload | null {
    try {
      return jwt.decode(token) as TokenPayload
    } catch (error) {
      return null
    }
  }

  /**
     * Get token expiry time
     */
  getTokenExpiry (token: string): Date | null {
    try {
      const decoded = jwt.decode(token) as any
      if (decoded && decoded.exp) {
        return new Date(decoded.exp * 1000)
      }
      return null
    } catch (error) {
      return null
    }
  }
}

export default new JWTService()
