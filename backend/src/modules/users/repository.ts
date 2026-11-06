import type { PoolClient } from 'pg'
import { BaseRepository } from '../../repositories/BaseRepository'
import type { CreateUserDTO, UpdateUserDTO, User } from './types'

class UserRepository extends BaseRepository<User> {
  constructor () {
    super('users')
  }

  /**
   * Create a new user
   */
  async create (data: CreateUserDTO, client?: PoolClient): Promise<User> {
    const { name, email, password, role } = data

    const fields: string[] = ['name', 'email']
    const values: unknown[] = [name, email]

    if (password) {
      fields.push('password')
      values.push(password)
    }

    if (role) {
      fields.push('role')
      values.push(role)
    }

    const placeholders = values.map((_, i) => `$${i + 1}`).join(', ')
    const sql = `INSERT INTO ${this.tableName} (${fields.join(', ')}) VALUES (${placeholders}) RETURNING *`

    if (client) {
      const result = await client.query(sql, values)
      return result.rows[0] as User
    }

    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, values)
      return result.rows[0] as User
    })
  }

  /**
   * Update a user by ID
   */
  async update (id: string | number, data: UpdateUserDTO, client?: PoolClient): Promise<User | null> {
    const fields: string[] = []
    const values: unknown[] = []
    let paramIndex = 1

    if (data.name !== undefined) {
      fields.push(`name = $${paramIndex++}`)
      values.push(data.name)
    }

    if (data.email !== undefined) {
      fields.push(`email = $${paramIndex++}`)
      values.push(data.email)
    }

    if (data.password !== undefined) {
      fields.push(`password = $${paramIndex++}`)
      values.push(data.password)
    }

    if (data.role !== undefined) {
      fields.push(`role = $${paramIndex++}`)
      values.push(data.role)
    }

    if (fields.length === 0) {
      return await this.findById(id, client)
    }

    fields.push('updated_at = NOW()')
    values.push(id)

    const sql = `UPDATE ${this.tableName} SET ${fields.join(', ')} WHERE id = $${paramIndex} RETURNING *`

    if (client) {
      const result = await client.query(sql, values)
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as User) : null
    }

    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, values)
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as User) : null
    })
  }

  /**
   * Find user by email
   */
  async findByEmail (email: string, client?: PoolClient): Promise<User | null> {
    return await this.findOneBy('email', email, client)
  }

  /**
   * Find users by role
   */
  async findByRole (role: string, client?: PoolClient): Promise<User[]> {
    return await this.findBy('role', role, client)
  }

  /**
   * Check if email exists
   */
  async emailExists (email: string, client?: PoolClient): Promise<boolean> {
    const sql = `SELECT EXISTS(SELECT 1 FROM ${this.tableName} WHERE email = $1) as exists`
    if (client) {
      const result = await client.query(sql, [email])
      return result.rows[0].exists
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(sql, [email])
      return result.rows[0].exists
    })
  }

  /**
   * Get user with pagination
   */
  async findAllPaginated (page: number = 1, limit: number = 10, client?: PoolClient) {
    const offset = (page - 1) * limit
    const execute = async (c: PoolClient) => {
      const dataResult = await c.query(
        `SELECT * FROM ${this.tableName} ORDER BY created_at DESC LIMIT $1 OFFSET $2`,
        [limit, offset]
      )
      const countResult = await c.query(`SELECT COUNT(*) as count FROM ${this.tableName}`)

      const total = parseInt(countResult.rows[0].count, 10)
      const totalPages = Math.ceil(total / limit)

      return {
        data: dataResult.rows as User[],
        total,
        page,
        limit,
        totalPages
      }
    }

    if (client) {
      return await execute(client)
    }
    return await this.withReadOnlyTransaction(execute)
  }

  /**
   * Search users by name or email
   */
  async search (searchTerm: string, client?: PoolClient): Promise<User[]> {
    const sql = `SELECT * FROM ${this.tableName} WHERE name ILIKE $1 OR email ILIKE $1 ORDER BY created_at DESC`
    const params = [`%${searchTerm}%`]

    if (client) {
      const result = await client.query(sql, params)
      return result.rows as User[]
    }

    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(sql, params)
      return result.rows as User[]
    })
  }
}

export default new UserRepository()