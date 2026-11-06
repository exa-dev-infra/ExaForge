import crypto from 'crypto'
import type { User } from '../users/types'
import userRepository from '../users/repository'
import refreshTokenRepository from './refreshTokenRepository'
import Database from '../../config/database'
import {
  EmailAlreadyExistsError,
  InvalidCredentialsError,
  InvalidCurrentPasswordError,
  InvalidRefreshTokenError,
  RefreshTokenExpiredError,
  UserNotFoundError,
  ValidationError,
  RESPONSE_CODES
} from '../../types/errors'
import jwtService from '../../services/jwtService'
import passwordService from '../../services/passwordService'
import type { AuthResponse, LoginDTO, RefreshTokenResponse, RegisterDTO } from './types'

export interface RequestMeta {
  userAgent?: string
  ipAddress?: string
}

function hashToken (token: string): string {
  return crypto.createHash('sha256').update(token).digest('hex')
}

class AuthService {
  private get rotationThresholdMs (): number {
    const days = parseInt(process.env.JWT_ROTATION_THRESHOLD_DAYS || '2', 10)
    return (Number.isNaN(days) ? 2 : days) * 24 * 60 * 60 * 1000
  }

  /**
   * Register a new user
   */
  async register (data: RegisterDTO, meta?: RequestMeta): Promise<AuthResponse> {
    const { name, email, password, role } = data

    // Validation
    if (!name || !email || !password) {
      throw new ValidationError('Name, email, and password are required', {
        fields: ['name', 'email', 'password']
      })
    }

    // Validate password strength
    const passwordValidation = passwordService.validateStrength(password)
    if (!passwordValidation.valid) {
      throw new ValidationError(
        'Password does not meet requirements',
        {
          field: 'password',
          errors: passwordValidation.errors
        },
        RESPONSE_CODES.PASSWORD_REQUIREMENTS_NOT_MET
      )
    }

    const db = Database.getInstance()

    const user = await db.withTransaction(async (client) => {
      // Check if user already exists
      const existingUser = await userRepository.findByEmail(email, client)
      if (existingUser) {
        throw new EmailAlreadyExistsError('Email already exists', { field: 'email' })
      }

      // Hash password
      const hashedPassword = await passwordService.hash(password)

      // Create user
      return await userRepository.create(
        {
          name,
          email,
          password: hashedPassword,
          role: role || 'user'
        },
        client
      )
    })

    // Generate tokens
    const tokens = jwtService.generateTokenPair({
      userId: user.id,
      email: user.email,
      role: user.role || 'user'
    })

    const expiresAt =
      jwtService.getTokenExpiry(tokens.refreshToken) ||
      new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)

    // Store refresh token in PostgreSQL database
    await refreshTokenRepository.create({
      userId: user.id,
      tokenHash: hashToken(tokens.refreshToken),
      expiresAt,
      userAgent: meta?.userAgent,
      ipAddress: meta?.ipAddress
    })

    return {
      user: {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role || 'user'
      },
      tokens
    }
  }

  /**
   * Login user
   */
  async login (data: LoginDTO, meta?: RequestMeta): Promise<AuthResponse> {
    const { email, password } = data

    // Validation
    if (!email || !password) {
      throw new ValidationError('Email and password are required', {
        fields: ['email', 'password']
      })
    }

    // Find user
    const user = await userRepository.findByEmail(email)

    if (!user || !user.password) {
      throw new InvalidCredentialsError('Invalid email or password')
    }

    // Verify password
    const isPasswordValid = await passwordService.compare(password, user.password)
    if (!isPasswordValid) {
      throw new InvalidCredentialsError('Invalid email or password')
    }

    // Generate tokens
    const tokens = jwtService.generateTokenPair({
      userId: user.id,
      email: user.email,
      role: user.role || 'user'
    })

    const expiresAt =
      jwtService.getTokenExpiry(tokens.refreshToken) ||
      new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)

    // Store refresh token in PostgreSQL database and update last_login atomically
    const db = Database.getInstance()
    await db.withTransaction(async (client) => {
      await refreshTokenRepository.create(
        {
          userId: user.id,
          tokenHash: hashToken(tokens.refreshToken),
          expiresAt,
          userAgent: meta?.userAgent,
          ipAddress: meta?.ipAddress
        },
        client
      )

      await client.query('UPDATE users SET last_login = NOW() WHERE id = $1', [user.id])
    })

    return {
      user: {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role || 'user'
      },
      tokens
    }
  }

  /**
   * Refresh access token with rotation threshold check
   */
  async refreshToken (rawRefreshToken: string, meta?: RequestMeta): Promise<RefreshTokenResponse> {
    if (!rawRefreshToken || !rawRefreshToken.trim()) {
      throw new ValidationError(
        'Refresh token is required',
        { field: 'refreshToken' },
        RESPONSE_CODES.REFRESH_TOKEN_REQUIRED
      )
    }

    // Verify refresh token signature & format
    const decoded = jwtService.verifyRefreshToken(rawRefreshToken)
    const tokenHash = hashToken(rawRefreshToken)

    // Check if refresh token exists in Database
    const storedToken = await refreshTokenRepository.findByTokenHash(tokenHash)
    if (!storedToken || storedToken.revoked_at !== null) {
      throw new InvalidRefreshTokenError('Invalid or revoked refresh token')
    }

    // Check expiration
    if (new Date(storedToken.expires_at).getTime() <= Date.now()) {
      throw new RefreshTokenExpiredError('Refresh token has expired')
    }

    // Get user data
    const user = await userRepository.findById(decoded.userId)
    if (!user) {
      throw new UserNotFoundError('User not found')
    }

    // Always issue a new access token
    const newAccessToken = jwtService.generateAccessToken({
      userId: user.id,
      email: user.email,
      role: user.role || 'user'
    })

    const expiresAtMs = new Date(storedToken.expires_at).getTime()
    const remainingMs = expiresAtMs - Date.now()
    const shouldRotate = remainingMs <= this.rotationThresholdMs

    let finalRefreshToken = rawRefreshToken

    if (shouldRotate) {
      // Generate new refresh token
      finalRefreshToken = jwtService.generateRefreshToken({
        userId: user.id,
        email: user.email,
        role: user.role || 'user'
      })

      const newExpiresAt =
        jwtService.getTokenExpiry(finalRefreshToken) ||
        new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)

      // Atomically revoke/delete old token and insert new rotated token
      const db = Database.getInstance()
      await db.withTransaction(async (client) => {
        await refreshTokenRepository.deleteByTokenHash(tokenHash, client)
        await refreshTokenRepository.create(
          {
            userId: user.id,
            tokenHash: hashToken(finalRefreshToken),
            expiresAt: newExpiresAt,
            userAgent: meta?.userAgent,
            ipAddress: meta?.ipAddress
          },
          client
        )
      })
    }

    return {
      user: {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role || 'user'
      },
      tokens: {
        accessToken: newAccessToken,
        refreshToken: finalRefreshToken
      },
      rotated: shouldRotate
    }
  }

  /**
   * Logout user (revokes/deletes refresh token)
   */
  async logout (userId?: number, rawRefreshToken?: string): Promise<void> {
    if (rawRefreshToken) {
      await refreshTokenRepository.deleteByTokenHash(hashToken(rawRefreshToken))
    } else if (userId) {
      await refreshTokenRepository.deleteAllUserTokens(userId)
    }
  }

  /**
   * Change password (invalidates all active sessions)
   */
  async changePassword (userId: number, oldPassword: string, newPassword: string): Promise<void> {
    // Validation
    if (!oldPassword || !newPassword) {
      throw new ValidationError('Old password and new password are required', {
        fields: ['oldPassword', 'newPassword']
      })
    }

    // Get user
    const user = await userRepository.findById(userId)
    if (!user || !user.password) {
      throw new UserNotFoundError('User not found')
    }

    // Verify old password
    const isPasswordValid = await passwordService.compare(oldPassword, user.password)
    if (!isPasswordValid) {
      throw new InvalidCurrentPasswordError('Invalid current password')
    }

    // Validate new password strength
    const passwordValidation = passwordService.validateStrength(newPassword)
    if (!passwordValidation.valid) {
      throw new ValidationError(
        'Password does not meet requirements',
        {
          field: 'newPassword',
          errors: passwordValidation.errors
        },
        RESPONSE_CODES.PASSWORD_REQUIREMENTS_NOT_MET
      )
    }

    // Hash new password
    const hashedPassword = await passwordService.hash(newPassword)

    const db = Database.getInstance()
    await db.withTransaction(async (client) => {
      // Update password
      await userRepository.update(userId, { password: hashedPassword }, client)

      // Invalidate all active sessions for this user
      await refreshTokenRepository.deleteAllUserTokens(userId, client)
    })
  }

  /**
   * Get current user profile
   */
  async getProfile (userId: number): Promise<User> {
    const user = await userRepository.findById(userId)

    if (!user) {
      throw new UserNotFoundError('User not found')
    }

    // Don't return password
    delete user.password

    return user
  }
}

export default new AuthService()