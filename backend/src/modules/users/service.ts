import type { CreateUserDTO, UpdateUserDTO, User } from './types'
import userRepository from './repository'
import refreshTokenRepository from '../auth/refreshTokenRepository'
import Database from '../../config/database'
import passwordService from '../../services/passwordService'
import { EmailAlreadyExistsError, UserNotFoundError, ValidationError } from '../../types/errors'

class UserService {
  /**
   * Get all users
   */
  async getAllUsers (): Promise<User[]> {
    return await userRepository.findAll()
  }

  /**
   * Get user by ID
   */
  async getUserById (id: string | number): Promise<User> {
    const user = await userRepository.findById(id)

    if (!user) {
      throw new UserNotFoundError()
    }

    return user
  }

  /**
   * Get users with pagination
   */
  async getUsersPaginated (page: number = 1, limit: number = 10) {
    return await userRepository.findAllPaginated(page, limit)
  }

  /**
   * Search users
   */
  async searchUsers (searchTerm: string): Promise<User[]> {
    if (!searchTerm || searchTerm.trim().length === 0) {
      throw new ValidationError('Search term is required', { field: 'q' })
    }

    return await userRepository.search(searchTerm)
  }

  /**
   * Create a new user (within an atomic database transaction)
   */
  async createUser (data: CreateUserDTO): Promise<User> {
    const { name, email, password } = data

    // Validation
    if (!name || !email) {
      throw new ValidationError('Name and email are required', {
        fields: ['name', 'email']
      })
    }

    const db = Database.getInstance()
    return await db.withTransaction(async (client) => {
      // Check if email already exists
      const existingUser = await userRepository.findByEmail(email, client)
      if (existingUser) {
        throw new EmailAlreadyExistsError('Email already exists', { field: 'email' })
      }

      // Hash password if provided
      let hashedPassword: string | undefined
      if (password) {
        hashedPassword = await passwordService.hash(password)
      }

      // Create user
      return await userRepository.create(
        {
          name,
          email,
          password: hashedPassword,
          role: data.role
        },
        client
      )
    })
  }

  /**
   * Update user (within an atomic database transaction)
   */
  async updateUser (id: string | number, data: UpdateUserDTO): Promise<User> {
    const { name, email, password, role } = data
    const db = Database.getInstance()

    return await db.withTransaction(async (client) => {
      // Check if user exists
      const existingUser = await userRepository.findById(id, client)
      if (!existingUser) {
        throw new UserNotFoundError()
      }

      // Check if email is being changed to an existing email
      if (email && email !== existingUser.email) {
        const emailExists = await userRepository.findByEmail(email, client)
        if (emailExists) {
          throw new EmailAlreadyExistsError('Email already exists', { field: 'email' })
        }
      }

      // Hash password if being updated
      let hashedPassword: string | undefined
      if (password) {
        hashedPassword = await passwordService.hash(password)
        // If password is updated, revoke active refresh tokens
        await refreshTokenRepository.deleteAllUserTokens(Number(id), client)
      }

      // If role changed, revoke existing active refresh tokens
      if (role && role !== existingUser.role) {
        await refreshTokenRepository.deleteAllUserTokens(Number(id), client)
      }

      // Update user
      const updatedUser = await userRepository.update(
        id,
        {
          name,
          email,
          password: hashedPassword,
          role: data.role
        },
        client
      )

      if (!updatedUser) {
        throw new UserNotFoundError()
      }

      return updatedUser
    })
  }

  /**
   * Hard delete user (within an atomic transaction that cleans up refresh tokens)
   */
  async deleteUser (id: string | number): Promise<void> {
    const db = Database.getInstance()

    await db.withTransaction(async (client) => {
      const existingUser = await userRepository.findById(id, client)
      if (!existingUser) {
        throw new UserNotFoundError()
      }

      // Hard delete user: atomically revoke/delete all active refresh tokens for this user
      await refreshTokenRepository.deleteAllUserTokens(Number(id), client)

      const deleted = await userRepository.delete(id, client)
      if (!deleted) {
        throw new UserNotFoundError()
      }
    })
  }

  /**
   * Get users by role
   */
  async getUsersByRole (role: string): Promise<User[]> {
    return await userRepository.findByRole(role)
  }

  /**
   * Check if email exists
   */
  async checkEmailExists (email: string): Promise<boolean> {
    return await userRepository.emailExists(email)
  }

  /**
   * Get user by email
   */
  async getUserByEmail (email: string): Promise<User | null> {
    return await userRepository.findByEmail(email)
  }
}

export default new UserService()