import type { NextFunction, Request, Response } from 'express'
import { appEnv } from '../config'
import { AppError, ConflictError, DatabaseError, ValidationError } from '../types/errors'
import { ApiResponse } from '../types/response'
import logger from '../utils/logger'
import {
  CHECK_CONSTRAINT_MAP,
  FOREIGN_KEY_CONSTRAINT_MAP,
  PG_ERROR_CODES,
  UNIQUE_CONSTRAINT_FIELD_MAP
} from '../constants/pgConstraints'

interface PostgresError extends Error {
  code?: string
  constraint?: string
  detail?: string
  table?: string
  column?: string
  routine?: string
}

export class ErrorHandler {
  /**
   * Handle PostgreSQL database errors
   */
  static handleDatabaseError (error: PostgresError): AppError {
    const code = error.code

    switch (code) {
    case PG_ERROR_CODES.UNIQUE_VIOLATION:
      return this.handleUniqueViolation(error)

    case PG_ERROR_CODES.FOREIGN_KEY_VIOLATION:
      return this.handleForeignKeyViolation(error)

    case PG_ERROR_CODES.NOT_NULL_VIOLATION:
      return this.handleNotNullViolation(error)

    case PG_ERROR_CODES.CHECK_VIOLATION:
      return this.handleCheckViolation(error)

    case PG_ERROR_CODES.INVALID_TEXT_REPRESENTATION:
      return new DatabaseError('Invalid data format provided', {
        column: error.column,
        detail: error.detail
      })

    case PG_ERROR_CODES.NUMERIC_VALUE_OUT_OF_RANGE:
      return new DatabaseError('Numeric value is out of range', {
        column: error.column
      })

    case PG_ERROR_CODES.UNDEFINED_TABLE:
      return new DatabaseError('Database table not found', {
        table: error.table
      })

    case PG_ERROR_CODES.UNDEFINED_COLUMN:
      return new DatabaseError('Database column not found', {
        column: error.column
      })

    default:
      return new DatabaseError('Database operation failed', {
        code: error.code,
        detail: error.detail
      })
    }
  }

  private static handleUniqueViolation (error: PostgresError): ConflictError {
    const constraint = error.constraint || ''
    const fieldName = UNIQUE_CONSTRAINT_FIELD_MAP[constraint] || constraint.replace(/_key$/, '')

    let message = `The ${fieldName} already exists`

    if (error.detail) {
      const match = error.detail.match(/Key \((.*?)\)=\((.*?)\)/)
      if (match) {
        const value = match[2]
        message = `The ${fieldName} '${value}' already exists`
      }
    }

    return new ConflictError(message, {
      field: fieldName,
      constraint,
      type: 'unique_violation'
    })
  }

  private static handleForeignKeyViolation (error: PostgresError): DatabaseError {
    const constraint = error.constraint || ''
    const fieldName = FOREIGN_KEY_CONSTRAINT_MAP[constraint] || constraint

    let message = `Invalid reference for ${fieldName}`

    if (error.detail?.includes('still referenced')) {
      message = 'Cannot delete: this record is still being referenced by other records'
    } else if (error.detail?.includes('not present')) {
      const match = error.detail.match(/Key \((.*?)\)=\((.*?)\)/)
      if (match) {
        const value = match[2]
        message = `Referenced ${fieldName} with value '${value}' does not exist`
      }
    }

    return new DatabaseError(message, {
      field: fieldName,
      constraint,
      type: 'foreign_key_violation',
      detail: error.detail
    })
  }

  private static handleNotNullViolation (error: PostgresError): DatabaseError {
    const column = error.column || 'unknown field'
    return new DatabaseError(`Field '${column}' is required and cannot be null`, {
      field: column,
      type: 'not_null_violation'
    })
  }

  private static handleCheckViolation (error: PostgresError): DatabaseError {
    const constraint = error.constraint || ''
    const fieldName = CHECK_CONSTRAINT_MAP[constraint] || constraint

    return new DatabaseError(`Invalid value for ${fieldName}`, {
      field: fieldName,
      constraint,
      type: 'check_violation',
      detail: error.detail
    })
  }

  static middleware (err: Error, req: Request, res: Response, _next: NextFunction): void {
    const { requestId } = req

    let error: AppError

    if (err instanceof AppError) {
      error = err
    } else if (typeof (err as AppError).code === 'string') {
      error = ErrorHandler.handleDatabaseError(err as PostgresError)
    } else {
      error = new AppError(
        err.message || 'An unexpected error occurred',
        500
      )
    }

    logger.error('request failed', {
      requestId,
      message: error.message,
      statusCode: error.statusCode,
      code: error.code,
      details: error.details,
      stack: error.stack
    })

    // Validation failures are a documented part of the API contract: the
    // client always receives the full list of field errors. All other kinds
    // of details are for development/debugging only.
    const includeDetails = error instanceof ValidationError || appEnv.isDevelopment

    const response = ApiResponse.error(
      error.message,
      error.code,
      includeDetails ? error.details : undefined
    )

    res.status(error.statusCode).json(response)
  }

  static notFound (req: Request, res: Response): void {
    const response = ApiResponse.notFound(`Route ${req.originalUrl} not found`)
    res.status(404).json(response)
  }

  static catchAsync (fn: (req: Request, res: Response, next: NextFunction) => unknown) {
    return (req: Request, res: Response, next: NextFunction): void => {
      Promise.resolve(fn(req, res, next)).catch(next)
    }
  }
}

export default ErrorHandler