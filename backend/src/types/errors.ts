import type { ResponseCode, HttpStatus } from '../constants/codes'
import { HTTP_STATUS, RESPONSE_CODES } from '../constants/codes'

export class AppError extends Error {
  public statusCode: HttpStatus
  public code: ResponseCode
  public isOperational: boolean
  public details?: unknown

  constructor (
    message: string,
    statusCode: HttpStatus = HTTP_STATUS.INTERNAL_SERVER_ERROR,
    code: ResponseCode = RESPONSE_CODES.INTERNAL_ERROR,
    isOperational = true,
    details?: unknown
  ) {
    super(message)
    this.statusCode = statusCode
    this.code = code
    this.isOperational = isOperational
    this.details = details

    Error.captureStackTrace(this, this.constructor)
  }
}

export class ValidationError extends AppError {
  constructor (message: string, details?: unknown, code: ResponseCode = RESPONSE_CODES.VALIDATION_ERROR) {
    super(message, HTTP_STATUS.BAD_REQUEST, code, true, details)
  }
}

export class BadRequestError extends AppError {
  constructor (message: string, details?: unknown, code: ResponseCode = RESPONSE_CODES.BAD_REQUEST) {
    super(message, HTTP_STATUS.BAD_REQUEST, code, true, details)
  }
}

export class UnauthorizedError extends AppError {
  constructor (message = 'Unauthorized', code: ResponseCode = RESPONSE_CODES.UNAUTHORIZED, details?: unknown) {
    super(message, HTTP_STATUS.UNAUTHORIZED, code, true, details)
  }
}

export class ForbiddenError extends AppError {
  constructor (message = 'Forbidden', code: ResponseCode = RESPONSE_CODES.FORBIDDEN, details?: unknown) {
    super(message, HTTP_STATUS.FORBIDDEN, code, true, details)
  }
}

export class NotFoundError extends AppError {
  constructor (message = 'Resource not found', code: ResponseCode = RESPONSE_CODES.NOT_FOUND, details?: unknown) {
    super(message, HTTP_STATUS.NOT_FOUND, code, true, details)
  }
}

export class ConflictError extends AppError {
  constructor (message: string, details?: unknown, code: ResponseCode = RESPONSE_CODES.CONFLICT) {
    super(message, HTTP_STATUS.CONFLICT, code, true, details)
  }
}

export class DatabaseError extends AppError {
  constructor (message: string, details?: unknown) {
    super(message, HTTP_STATUS.INTERNAL_SERVER_ERROR, RESPONSE_CODES.DATABASE_ERROR, true, details)
  }
}

export class ServiceUnavailableError extends AppError {
  constructor (message = 'Service unavailable') {
    super(message, HTTP_STATUS.SERVICE_UNAVAILABLE, RESPONSE_CODES.SERVICE_UNAVAILABLE, true)
  }
}

// ---- Dedicated Domain Semantic Error Classes ----

export class UserNotFoundError extends NotFoundError {
  constructor (message = 'User not found') {
    super(message, RESPONSE_CODES.USER_NOT_FOUND)
  }
}

export class InvalidCredentialsError extends UnauthorizedError {
  constructor (message = 'Invalid email or password') {
    super(message, RESPONSE_CODES.INVALID_CREDENTIALS)
  }
}

export class EmailAlreadyExistsError extends ConflictError {
  constructor (message = 'Email already exists', details?: unknown) {
    super(message, details, RESPONSE_CODES.EMAIL_ALREADY_EXISTS)
  }
}

export class InvalidCurrentPasswordError extends UnauthorizedError {
  constructor (message = 'Invalid current password') {
    super(message, RESPONSE_CODES.INVALID_CURRENT_PASSWORD)
  }
}

export class InvalidRefreshTokenError extends UnauthorizedError {
  constructor (message = 'Invalid or revoked refresh token') {
    super(message, RESPONSE_CODES.INVALID_REFRESH_TOKEN)
  }
}

export class RefreshTokenExpiredError extends UnauthorizedError {
  constructor (message = 'Refresh token has expired') {
    super(message, RESPONSE_CODES.REFRESH_TOKEN_EXPIRED)
  }
}

export class AccessTokenExpiredError extends UnauthorizedError {
  constructor (message = 'Access token has expired') {
    super(message, RESPONSE_CODES.ACCESS_TOKEN_EXPIRED)
  }
}

export class InvalidAccessTokenError extends UnauthorizedError {
  constructor (message = 'Invalid access token') {
    super(message, RESPONSE_CODES.INVALID_ACCESS_TOKEN)
  }
}

export class IdempotencyKeyRequiredError extends BadRequestError {
  constructor (message = 'Idempotency-Key header is required') {
    super(message, undefined, RESPONSE_CODES.IDEMPOTENCY_KEY_REQUIRED)
  }
}

export class IdempotencyConflictError extends ConflictError {
  constructor (message = 'A request with this idempotency key is currently being processed') {
    super(message, undefined, RESPONSE_CODES.IDEMPOTENCY_CONFLICT)
  }
}

export class IdempotencyPayloadMismatchError extends AppError {
  constructor (message = 'Idempotency key payload does not match the original request') {
    super(message, HTTP_STATUS.UNPROCESSABLE_ENTITY, RESPONSE_CODES.IDEMPOTENCY_PAYLOAD_MISMATCH, true)
  }
}

export { HTTP_STATUS, RESPONSE_CODES }
export type { ResponseCode, HttpStatus }