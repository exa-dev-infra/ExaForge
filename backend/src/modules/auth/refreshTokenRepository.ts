import type { PoolClient } from 'pg'
import { BaseRepository } from '../../repositories/BaseRepository'

export interface RefreshTokenRecord {
  id: number
  user_id: number
  token_hash: string
  expires_at: Date
  revoked_at: Date | null
  user_agent: string | null
  ip_address: string | null
  created_at: Date
  updated_at: Date
}

export interface CreateRefreshTokenDTO {
  userId: number
  tokenHash: string
  expiresAt: Date
  userAgent?: string
  ipAddress?: string
}

class RefreshTokenRepository extends BaseRepository<RefreshTokenRecord> {
  constructor () {
    super('refresh_tokens')
  }

  async create (
    data: Partial<RefreshTokenRecord> & Partial<CreateRefreshTokenDTO>,
    client?: PoolClient
  ): Promise<RefreshTokenRecord> {
    const userId = data.userId ?? data.user_id
    const tokenHash = data.tokenHash ?? data.token_hash
    const expiresAt = data.expiresAt ?? data.expires_at
    const userAgent = data.userAgent ?? data.user_agent
    const ipAddress = data.ipAddress ?? data.ip_address

    if (!userId || !tokenHash || !expiresAt) {
      throw new Error('userId, tokenHash, and expiresAt are required to create a refresh token')
    }

    const sql = `
      INSERT INTO ${this.tableName} (user_id, token_hash, expires_at, user_agent, ip_address)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING *
    `
    const values = [userId, tokenHash, expiresAt, userAgent || null, ipAddress || null]

    if (client) {
      const result = await client.query(sql, values)
      return result.rows[0] as RefreshTokenRecord
    }

    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, values)
      return result.rows[0] as RefreshTokenRecord
    })
  }

  async update (
    id: string | number,
    data: Partial<RefreshTokenRecord>,
    client?: PoolClient
  ): Promise<RefreshTokenRecord | null> {
    const fields: string[] = []
    const values: unknown[] = []
    let paramIndex = 1

    if (data.revoked_at !== undefined) {
      fields.push(`revoked_at = $${paramIndex++}`)
      values.push(data.revoked_at)
    }

    if (data.expires_at !== undefined) {
      fields.push(`expires_at = $${paramIndex++}`)
      values.push(data.expires_at)
    }

    if (fields.length === 0) {
      return await this.findById(id, client)
    }

    fields.push('updated_at = NOW()')
    values.push(id)

    const sql = `UPDATE ${this.tableName} SET ${fields.join(', ')} WHERE id = $${paramIndex} RETURNING *`

    if (client) {
      const result = await client.query(sql, values)
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as RefreshTokenRecord) : null
    }

    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, values)
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as RefreshTokenRecord) : null
    })
  }

  async findByTokenHash (tokenHash: string, client?: PoolClient): Promise<RefreshTokenRecord | null> {
    const sql = `SELECT * FROM ${this.tableName} WHERE token_hash = $1 LIMIT 1`
    if (client) {
      const result = await client.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as RefreshTokenRecord) : null
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as RefreshTokenRecord) : null
    })
  }

  async revokeToken (tokenHash: string, client?: PoolClient): Promise<boolean> {
    const sql = `UPDATE ${this.tableName} SET revoked_at = NOW(), updated_at = NOW() WHERE token_hash = $1`
    if (client) {
      const result = await client.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0
    }
    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0
    })
  }

  async revokeAllUserTokens (userId: number, client?: PoolClient): Promise<number> {
    const sql = `
      UPDATE ${this.tableName}
      SET revoked_at = NOW(), updated_at = NOW()
      WHERE user_id = $1 AND revoked_at IS NULL
    `
    if (client) {
      const result = await client.query(sql, [userId])
      return result.rowCount ?? 0
    }
    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, [userId])
      return result.rowCount ?? 0
    })
  }

  async deleteAllUserTokens (userId: number, client?: PoolClient): Promise<number> {
    const sql = `DELETE FROM ${this.tableName} WHERE user_id = $1`
    if (client) {
      const result = await client.query(sql, [userId])
      return result.rowCount ?? 0
    }
    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, [userId])
      return result.rowCount ?? 0
    })
  }

  async deleteByTokenHash (tokenHash: string, client?: PoolClient): Promise<boolean> {
    const sql = `DELETE FROM ${this.tableName} WHERE token_hash = $1`
    if (client) {
      const result = await client.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0
    }
    return await this.withTransaction(async (c) => {
      const result = await c.query(sql, [tokenHash])
      return (result.rowCount ?? 0) > 0
    })
  }
}

export default new RefreshTokenRepository()
