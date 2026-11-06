---
name: typescript-backend
description: >-
  Express TypeScript backend architecture, feature module pattern, JWT refresh token rotation with cookies & PostgreSQL, Admin CRUD with DB transactions, Zod validation boundaries, and ApiResponse envelope. Use when adding routes, services, repositories, or migrations.
---

# TypeScript Express Backend Skill Guide

This skill provides standard procedures, rules, and best practices for developing and maintaining the **TypeScript + Express** backend template.

---

## 1. Stack & Architecture

- **Runtime & Language**: Node.js 22+ / TypeScript (Strict mode, CommonJS).
- **Framework**: Express REST API.
- **Pattern**: Clean Architecture with feature-based modules under `src/modules/<feature>/`.

```text
src/
├── app.ts                  # Application entry, middleware, resource lifecycle
├── config/                 # Env configuration, DB/Redis/RabbitMQ clients
├── constants/              # RESPONSE_CODES, pgConstraints
├── repositories/           # BaseRepository.ts (supports optional PoolClient for composable transactions)
├── services/               # Shared infra services (jwt, password, realtime)
├── workers/                # RabbitMQ queue consumers
├── utils/                  # cookie.ts, validator.ts, logger.ts
└── modules/                # Domain feature modules
    ├── index.ts            # Feature registry & mountModules(app)
    ├── auth/               # Standard JWT auth, DB refresh tokens, cookie handling
    └── users/              # Standard Admin User CRUD with transactions & RBAC
```

---

## 2. Core Rules & Conventions

### A. Route Validation Boundary (Zod)
- All incoming requests must be validated using `validate()` middleware from `src/utils/validator.ts`.
- Always read data from `req.validated?.<source>`.

### B. Standard API Responses & Mandatory Semantic Error Codes
- Gunakan method `ApiResponse` dari `src/types/response.ts` (`ApiResponse.success`, `ApiResponse.created`, `ApiResponse.paginated`, etc.).
- **Wajib Menggunakan Semantic Error Codes**: Dilarang melempar generic error tanpa semantic code untuk kondisi domain spesifik. Selalu gunakan error classes semantik atau sertakan code dari `RESPONSE_CODES` (`src/constants/codes.ts`):
  - `UserNotFoundError` (`USER_NOT_FOUND`) alih-alih generic `NotFoundError`
  - `InvalidCredentialsError` (`INVALID_CREDENTIALS`) alih-alih generic `UnauthorizedError`
  - `EmailAlreadyExistsError` (`EMAIL_ALREADY_EXISTS`) alih-alih generic `ConflictError`
  - `InvalidCurrentPasswordError` (`INVALID_CURRENT_PASSWORD`)
  - `InvalidRefreshTokenError` (`INVALID_REFRESH_TOKEN`)
  - `RefreshTokenExpiredError` (`REFRESH_TOKEN_EXPIRED`)
  - `AccessTokenExpiredError` (`ACCESS_TOKEN_EXPIRED`)
  - `InvalidAccessTokenError` (`INVALID_ACCESS_TOKEN`)
- **Pesan Error Wajib Bahasa Inggris Default**: Semua pesan bawaan backend wajib ditulis dalam bahasa Inggris (misal `"User not found"`, `"Invalid email or password"`). Lokalisasi i18n (misal bahasa Indonesia) ditangani oleh client-side melalui mapping `errors.${apiErr.code}`.
- Bungkus router handlers dengan `ErrorHandler.catchAsync`.

### C. Standard Token Flow & Refresh Token Rotation
- **Hybrid Delivery (Web + Mobile Support)**:
  - `access_token`: Cookie (`httpOnly: false`, path: `/`, maxAge: 15m) + returned in JSON response body.
  - `refresh_token`: Cookie (`httpOnly: true`, path: `/`, maxAge: 7 days) + returned in JSON response body.
- **Database Storage**:
  - Refresh tokens are hashed (SHA-256) and saved in the PostgreSQL `refresh_tokens` table.
- **Rotation Window**:
  - Configurable via `JWT_ROTATION_THRESHOLD_DAYS` (default: 2 days).
  - When `/auth/refresh` is called:
    - New access token is **always** issued.
    - If remaining lifetime `<= threshold` (misal <= 2 hari), refresh token lama di-revoke/dihapus dan refresh token baru di-generate (+7 hari) di DB & cookie.
    - If remaining lifetime `> threshold`, refresh token lama dipertahankan.
  - Endpoint `/auth/refresh` mengecek `req.cookies.refresh_token` terlebih dahulu, fallback ke `req.body.refreshToken` (untuk mobile app).

### D. Standard CRUD User/Admin & Database Transactions
- Seluruh endpoint manajemen pengguna (`/users`) **WAJIB** dilindungi middleware `authenticate` dan `authorize('admin')`.
- Service layer mengelola batas transaksi atomik melalui `db.withTransaction(async (client) => { ... })`:
  - `createUser`: Atomik pengecekan email + insert user.
  - `updateUser`: Atomik update profil + pembatalan refresh token jika password atau role diubah.
  - `deleteUser`: Atomik penghapusan seluruh active refresh tokens di tabel `refresh_tokens` + hard delete user di tabel `users`.
- Repository methods menerima parameter opsional `client?: PoolClient` agar dapat berpartisipasi dalam transaksi yang sama.

---

## 3. Quality Gate & Mandatory Commands

```bash
# Type check (MUST PASS)
npm run typecheck

# Lint check & auto-fix (MUST PASS with 0 errors)
npm run lint

# Build production dist/
npm run build

# Unit test suite (MUST PASS)
npm test

# Run migrations
npm run migrate:up
```
