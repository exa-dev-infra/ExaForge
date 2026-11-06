import type { ResponseCode } from '../constants/codes'
import { HTTP_STATUS, RESPONSE_CODES } from '../constants/codes'

// Every HTTP response follows this shape. The `code` and `requestId` fields
// are mandatory at runtime: `code` is always taken from RESPONSE_CODES and
// `requestId` is injected by the request-id middleware.

export type BaseResponse<T = unknown> = {
  success: boolean
  message: string
  code: ResponseCode
  data: T | null
  timestamp: string
  requestId?: string
}

export interface PaginationInfo {
  page: number
  limit: number
  total: number
  totalPages: number
}

export interface PaginatedResponse<T> extends BaseResponse<T> {
  pagination: PaginationInfo
}

export class ApiResponse {
  static success<T> (data: T, message = 'Success'): BaseResponse<T> {
    return {
      success: true,
      message,
      data,
      code: RESPONSE_CODES.SUCCESS,
      timestamp: new Date().toISOString()
    }
  }

  static created<T> (data: T, message = 'Resource created successfully'): BaseResponse<T> {
    return {
      success: true,
      message,
      data,
      code: RESPONSE_CODES.CREATED,
      timestamp: new Date().toISOString()
    }
  }

  static updated<T> (data: T, message = 'Resource updated successfully'): BaseResponse<T> {
    return {
      success: true,
      message,
      data,
      code: RESPONSE_CODES.UPDATED,
      timestamp: new Date().toISOString()
    }
  }

  static deleted (message = 'Resource deleted successfully'): BaseResponse<unknown> {
    return {
      success: true,
      message,
      data: null,
      code: RESPONSE_CODES.DELETED,
      timestamp: new Date().toISOString()
    }
  }

  static error (
    message: string,
    code: ResponseCode = RESPONSE_CODES.INTERNAL_ERROR,
    data: unknown = null
  ): BaseResponse<unknown> {
    return {
      success: false,
      message,
      data,
      code,
      timestamp: new Date().toISOString()
    }
  }

  static notFound (message = 'Resource not found'): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.NOT_FOUND)
  }

  static badRequest (message = 'Bad request', data: unknown = null): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.BAD_REQUEST, data)
  }

  static validationError (message: string, data: unknown = null): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.VALIDATION_ERROR, data)
  }

  static unauthorized (message = 'Unauthorized'): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.UNAUTHORIZED)
  }

  static forbidden (message = 'Forbidden'): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.FORBIDDEN)
  }

  static conflict (message = 'Resource already exists', data: unknown = null): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.CONFLICT, data)
  }

  static databaseError (message = 'Database error', data: unknown = null): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.DATABASE_ERROR, data)
  }

  static serviceUnavailable (message = 'Service unavailable'): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.SERVICE_UNAVAILABLE)
  }

  static internalError (message = 'Internal server error', data: unknown = null): BaseResponse<unknown> {
    return this.error(message, RESPONSE_CODES.INTERNAL_ERROR, data)
  }

  static paginated<T> (
    data: T[],
    page: number,
    limit: number,
    total: number,
    message = 'Success'
  ): PaginatedResponse<T[]> {
    const totalPages = total === 0 ? 0 : Math.ceil(total / limit)
    const base = this.success(data, message) as PaginatedResponse<T[]>

    base.pagination = {
      page,
      limit,
      total,
      totalPages
    }

    return base
  }
}

export { HTTP_STATUS }