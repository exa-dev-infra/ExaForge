import type { PoolClient, PoolConfig } from 'pg'
import { Pool } from 'pg'
import dotenv from 'dotenv'
import logger from '../utils/logger'
import { isRequestAborted } from '../utils/requestContext'
import { AppError } from '../types/errors'

dotenv.config()

const poolConfig: PoolConfig = process.env.DATABASE_URL
  ? {
    connectionString: process.env.DATABASE_URL,
    max: 20,
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 2000
  }
  : {
    host: process.env.DATABASE_HOST || 'localhost',
    port: parseInt(process.env.DATABASE_PORT || '5432'),
    database: process.env.DATABASE_NAME || 'mydb',
    user: process.env.DATABASE_USER || 'postgres',
    password: process.env.DATABASE_PASSWORD || 'password',
    max: 20, // Maximum number of clients in the pool
    idleTimeoutMillis: 30000, // Close idle clients after 30 seconds
    connectionTimeoutMillis: 2000 // Return an error after 2 seconds if connection could not be established
  }

class Database {
  private pool: Pool
  private static instance: Database

  private constructor () {
    this.pool = new Pool(poolConfig)

    // Handle pool errors
    this.pool.on('error', (err) => {
      logger.error('Unexpected error on idle client', { err })
      process.exit(-1)
    })
  }

  public static getInstance (): Database {
    if (!Database.instance) {
      Database.instance = new Database()
    }
    return Database.instance
  }

  public getPool (): Pool {
    return this.pool
  }

  public async query (text: string, params?: unknown[]) {
    if (isRequestAborted()) {
      logger.warn('Skipping query execution, request was aborted by client', { text })
      throw new AppError('Client aborted request', 499 as any, 'CLIENT_CLOSED_REQUEST' as any)
    }

    const start = Date.now()
    try {
      const res = await this.pool.query(text, params)
      const duration = Date.now() - start
      logger.info('Executed query', { text, duration, rows: res.rowCount })
      return res
    } catch (error) {
      logger.error('Database query error:', { error })
      throw error
    }
  }

  public async getClient () {
    return await this.pool.connect()
  }

  /**
     * Execute a function within a database transaction
     * Automatically handles BEGIN, COMMIT, and ROLLBACK
     *
     * @example
     * await db.withTransaction(async (client) => {
     *   await client.query('INSERT INTO users (name) VALUES ($1)', ['John']);
     *   await client.query('INSERT INTO logs (action) VALUES ($1)', ['user_created']);
     * });
     */
  public async withTransaction<T> (fn: (client: PoolClient) => Promise<T>): Promise<T> {
    if (isRequestAborted()) {
      logger.warn('Skipping transaction, request was aborted by client')
      throw new AppError('Client aborted request', 499 as any, 'CLIENT_CLOSED_REQUEST' as any)
    }

    const client = await this.pool.connect()

    try {
      await client.query('BEGIN')
      logger.info('Transaction started')

      const result = await fn(client)

      await client.query('COMMIT')
      logger.info('Transaction committed')

      return result
    } catch (error) {
      try {
        await client.query('ROLLBACK')
        logger.info('Transaction rolled back')
      } catch (rollbackError) {
        logger.error('Failed to rollback transaction:', { rollbackError })
      }

      // Re-throw the original error
      throw error
    } finally {
      client.release()
    }
  }

  /**
   * Execute a function within a read-only database transaction
   */
  public async withReadOnlyTransaction<T> (fn: (client: PoolClient) => Promise<T>): Promise<T> {
    if (isRequestAborted()) {
      logger.warn('Skipping read-only transaction, request was aborted by client')
      throw new AppError('Client aborted request', 499 as any, 'CLIENT_CLOSED_REQUEST' as any)
    }

    const client = await this.pool.connect()

    try {
      await client.query('BEGIN TRANSACTION READ ONLY')
      logger.info('Read-only transaction started')

      const result = await fn(client)

      await client.query('COMMIT')
      logger.info('Read-only transaction committed')

      return result
    } catch (error) {
      try {
        await client.query('ROLLBACK')
        logger.info('Read-only transaction rolled back')
      } catch (rollbackError) {
        logger.error('Failed to rollback read-only transaction:', { rollbackError })
      }
      throw error
    } finally {
      client.release()
    }
  }

  public async close () {
    await this.pool.end()
  }
}

export default Database
