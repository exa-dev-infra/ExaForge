---
description: Mandatory semantic error codes, English default messages, and frontend i18n locale synchronization
trigger: glob
globs: "**/*.ts"
---

# Error Handling & Localization Rules

All backend error returns and frontend error representations must strictly adhere to the following rules:

---

## 1. Mandatory Semantic Error Codes on Backend Error Returns

In all backend templates (Express, Go Fiber, FastAPI, Spring Boot), the frontend error resolution pipeline maps errors directly to translation keys using `errors.${apiErr.code}`.

1. **Never return bare generic errors for specific domain conditions**:
   - ❌ **Forbidden**:
     ```typescript
     // Bad: code defaults to generic "BAD_REQUEST" or "NOT_FOUND", frontend cannot localize the specific issue
     throw new NotFoundError('User not found')
     throw new UnauthorizedError('Invalid credentials')
     throw new ValidationError('Email already exists')
     ```
   - ✅ **Required**:
     ```typescript
     // Good: explicit semantic error code from RESPONSE_CODES or dedicated semantic error classes
     throw new UserNotFoundError() // code: 'USER_NOT_FOUND'
     throw new InvalidCredentialsError() // code: 'INVALID_CREDENTIALS'
     throw new EmailAlreadyExistsError('Email already exists', { field: 'email' }) // code: 'EMAIL_ALREADY_EXISTS'
     // Or using explicit code parameter:
     throw new NotFoundError('User not found', RESPONSE_CODES.USER_NOT_FOUND)
     throw new UnauthorizedError('Invalid email or password', RESPONSE_CODES.INVALID_CREDENTIALS)
     throw new ValidationError('Refresh token is required', { field: 'refreshToken' }, RESPONSE_CODES.REFRESH_TOKEN_REQUIRED)
     ```

2. **Use Semantic Error Classes or Dedicated Constructors**:
   - `UserNotFoundError` (`USER_NOT_FOUND`)
   - `InvalidCredentialsError` (`INVALID_CREDENTIALS`)
   - `EmailAlreadyExistsError` (`EMAIL_ALREADY_EXISTS`)
   - `InvalidCurrentPasswordError` (`INVALID_CURRENT_PASSWORD`)
   - `InvalidRefreshTokenError` (`INVALID_REFRESH_TOKEN`)
   - `RefreshTokenExpiredError` (`REFRESH_TOKEN_EXPIRED`)
   - `AccessTokenExpiredError` (`ACCESS_TOKEN_EXPIRED`)
   - `InvalidAccessTokenError` (`INVALID_ACCESS_TOKEN`)
   - Or with generic constructors:
     - `new NotFoundError(message, RESPONSE_CODES.USER_NOT_FOUND)`
     - `new UnauthorizedError(message, RESPONSE_CODES.INVALID_CREDENTIALS)`
     - `new ConflictError(message, { field }, RESPONSE_CODES.EMAIL_ALREADY_EXISTS)`
     - `new ValidationError(message, details, RESPONSE_CODES.PASSWORD_REQUIREMENTS_NOT_MET)`

---

## 2. Default Error Messages in Backend Must Be in English

All error messages defined on the backend (the `message` field of `AppError`, database errors, validation messages, and success responses) **must be written in English by default**:

- ❌ **Forbidden**:
  ```typescript
  throw new UserNotFoundError('User tidak ditemukan')
  throw new InvalidCredentialsError('Email atau password salah')
  ```
- ✅ **Required**:
  ```typescript
  throw new UserNotFoundError('User not found')
  throw new InvalidCredentialsError('Invalid email or password')
  ```

Localization to other languages (such as Indonesian) is strictly handled on the client-side via the frontend translation dictionary (`frontend/src/i18n/locales/id/errors.ts`) using the semantic `code`.

---

## 3. Centralized Error Code Constants

All error codes must be declared in a centralized constant registry (e.g. `src/constants/codes.ts` in Express, `pkg/constants/codes.go` in Go, `core/constants/codes.py` in FastAPI, `constant/ResponseCode.java` in Spring Boot):

- Use `SCREAMING_SNAKE_CASE` string values:
  ```typescript
  export const RESPONSE_CODES = {
    // Domain Auth & Token
    INVALID_CREDENTIALS: 'INVALID_CREDENTIALS',
    REFRESH_TOKEN_REQUIRED: 'REFRESH_TOKEN_REQUIRED',
    REFRESH_TOKEN_EXPIRED: 'REFRESH_TOKEN_EXPIRED',
    INVALID_REFRESH_TOKEN: 'INVALID_REFRESH_TOKEN',
    ACCESS_TOKEN_EXPIRED: 'ACCESS_TOKEN_EXPIRED',
    INVALID_ACCESS_TOKEN: 'INVALID_ACCESS_TOKEN',
    TOKEN_VERIFICATION_FAILED: 'TOKEN_VERIFICATION_FAILED',
    PASSWORD_REQUIREMENTS_NOT_MET: 'PASSWORD_REQUIREMENTS_NOT_MET',
    INVALID_CURRENT_PASSWORD: 'INVALID_CURRENT_PASSWORD',
    ACCOUNT_DEACTIVATED: 'ACCOUNT_DEACTIVATED',

    // Domain User
    USER_NOT_FOUND: 'USER_NOT_FOUND',
    EMAIL_ALREADY_EXISTS: 'EMAIL_ALREADY_EXISTS',
    USER_ALREADY_EXISTS: 'USER_ALREADY_EXISTS'
  } as const
  ```

---

## 4. Verification Checklist

Before completing any task that introduces or modifies error returns:
1. Every error return / throw in the backend code uses an explicit semantic code from the constants registry.
2. All default messages on the backend are in English.
3. If connected to a frontend with i18n, both `id/errors.ts` and `en/errors.ts` contain the matching translation key.
4. Backend unit tests pass and assert the semantic `code`.
