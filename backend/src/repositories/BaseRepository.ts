import type { Pool, PoolClient, QueryResult } from 'pg'
import Database from '../config/database'

export interface IBaseRepository<T> {
  findAll(client?: PoolClient): Promise<T[]>
  findById(id: string | number, client?: PoolClient): Promise<T | null>
  create(data: Partial<T>, client?: PoolClient): Promise<T>
  update(id: string | number, data: Partial<T>, client?: PoolClient): Promise<T | null>
  delete(id: string | number, client?: PoolClient): Promise<boolean>
}

export abstract class BaseRepository<T> implements IBaseRepository<T> {
  protected db: Database
  protected pool: Pool
  protected tableName: string

  constructor (tableName: string) {
    this.db = Database.getInstance()
    this.pool = this.db.getPool()
    this.tableName = tableName
  }

  /**
   * Get all records from the table
   */
  async findAll (client?: PoolClient): Promise<T[]> {
    if (client) {
      const result = await client.query(
        `SELECT * FROM ${this.tableName} ORDER BY created_at DESC`
      )
      return result.rows as T[]
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(
        `SELECT * FROM ${this.tableName} ORDER BY created_at DESC`
      )
      return result.rows as T[]
    })
  }

  /**
   * Find a record by ID
   */
  async findById (id: string | number, client?: PoolClient): Promise<T | null> {
    if (client) {
      const result = await client.query(
        `SELECT * FROM ${this.tableName} WHERE id = $1`,
        [id]
      )
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as T) : null
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(
        `SELECT * FROM ${this.tableName} WHERE id = $1`,
        [id]
      )
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as T) : null
    })
  }

  /**
   * Find records by custom condition
   */
  async findBy (field: string, value: unknown, client?: PoolClient): Promise<T[]> {
    if (client) {
      const result = await client.query(
        `SELECT * FROM ${this.tableName} WHERE ${field} = $1`,
        [value]
      )
      return result.rows as T[]
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(
        `SELECT * FROM ${this.tableName} WHERE ${field} = $1`,
        [value]
      )
      return result.rows as T[]
    })
  }

  /**
   * Find one record by custom condition
   */
  async findOneBy (field: string, value: unknown, client?: PoolClient): Promise<T | null> {
    if (client) {
      const result = await client.query(
        `SELECT * FROM ${this.tableName} WHERE ${field} = $1 LIMIT 1`,
        [value]
      )
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as T) : null
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(
        `SELECT * FROM ${this.tableName} WHERE ${field} = $1 LIMIT 1`,
        [value]
      )
      return (result.rowCount ?? 0) > 0 ? (result.rows[0] as T) : null
    })
  }

  /**
   * Create a new record
   */
  abstract create(data: Partial<T>, client?: PoolClient): Promise<T>

  /**
   * Update a record by ID
   */
  abstract update(id: string | number, data: Partial<T>, client?: PoolClient): Promise<T | null>

  /**
   * Delete a record by ID
   */
  async delete (id: string | number, client?: PoolClient): Promise<boolean> {
    if (client) {
      const result = await client.query(
        `DELETE FROM ${this.tableName} WHERE id = $1 RETURNING *`,
        [id]
      )
      return (result.rowCount ?? 0) > 0
    }
    return await this.withTransaction(async (c) => {
      const result = await c.query(
        `DELETE FROM ${this.tableName} WHERE id = $1 RETURNING *`,
        [id]
      )
      return (result.rowCount ?? 0) > 0
    })
  }

  /**
   * Count records
   */
  async count (client?: PoolClient): Promise<number> {
    if (client) {
      const result = await client.query(`SELECT COUNT(*) as count FROM ${this.tableName}`)
      return parseInt(result.rows[0].count, 10)
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(`SELECT COUNT(*) as count FROM ${this.tableName}`)
      return parseInt(result.rows[0].count, 10)
    })
  }

  /**
   * Check if a record exists
   */
  async exists (id: string | number, client?: PoolClient): Promise<boolean> {
    if (client) {
      const result = await client.query(
        `SELECT EXISTS(SELECT 1 FROM ${this.tableName} WHERE id = $1) as exists`,
        [id]
      )
      return result.rows[0].exists
    }
    return await this.withReadOnlyTransaction(async (c) => {
      const result = await c.query(
        `SELECT EXISTS(SELECT 1 FROM ${this.tableName} WHERE id = $1) as exists`,
        [id]
      )
      return result.rows[0].exists
    })
  }

  /**
   * Execute a custom query
   */
  protected async query (text: string, params?: unknown[]): Promise<QueryResult> {
    return await this.db.query(text, params)
  }

  /**
   * Execute within a transaction
   */
  async withTransaction<R> (fn: (client: PoolClient) => Promise<R>): Promise<R> {
    return await this.db.withTransaction(fn)
  }

  /**
   * Execute within a read-only transaction
   */
  async withReadOnlyTransaction<R> (fn: (client: PoolClient) => Promise<R>): Promise<R> {
    return await this.db.withReadOnlyTransaction(fn)
  }
}
