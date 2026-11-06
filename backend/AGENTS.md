# AGENTS.md

Project rules for AI coding agents (opencode, Claude Code, Antigravity/jules, etc.).
Follow everything below for every task in this repo.

## Project

- Node.js 22+ / TypeScript (strict, CommonJS) + Express REST API backend.
- Modules: PostgreSQL (pg Pool), Redis, RabbitMQ, node-pg-migrate migrations,
  multer uploads, **Zod validation**, structured logger, SSE/WebSocket realtime.
- Entry point `src/app.ts`; build output `dist/`.

## Commands

- `npm run typecheck` — `tsc --noEmit` — MUST pass before finishing any task.
- `npm run lint` / `npm run lint:fix` — `eslint .` — MUST pass; run `--fix` if needed.
- `npm run build` — `tsc` produces `dist/`.
- `npm run dev` — ts-node-dev with hot reload. `npm start` — runs `dist/app.js`.
- `npm run migrate:up|down|create` — node-pg-migrate, migrations in `migrations/`.
- `npm run docker:up|down` — starts/stops Postgres+Redis+RabbitMQ.
- `npm run workers` — run email/notification workers.
- `npm test` — integration tests (`test/`), run via tsx + node:test + supertest.
- Dev smoke: `npm run docker:up`, wait ~10s, `npm run migrate:up`, `npm run dev`.

## Architecture (feature modules — never break it)

```
src/
├── app.ts                  entry, wires global middleware + resource lifecycle
├── config/                env config + DB/Redis/RabbitMQ clients
├── constants/  middleware/  types/  utils/   # shared infra
├── repositories/BaseRepository.ts   # shared base for all repositories
├── services/              SHARED infra services (jwt, password, cache, queue,
│                          realtime, sse, websocket) — NOT feature code
├── workers/               RabbitMQ queue consumers
└── modules/               feature modules
    ├── index.ts           feature registry + mountModules(app)
    ├── <feature>/
    │   ├── routes.ts       Express router (controllers inline)
    │   ├── service.ts      business logic, singleton object (default export)
    │   ├── repository.ts   only when DB-backed; extends BaseRepository
    │   ├── types.ts        domain types / DTOs
    │   └── validation.ts   Zod schemas for each route
```

Request flow per feature: `HTTP → module routes.ts → validate() boundary
(req.validated) → service.ts → repository.ts → database.ts pool`.

- Register every feature in `src/modules/index.ts` (`featureModules[]`); it is
  mounted by `mountModules(app)` in `app.ts`. Flag-gated features set
  `enabled` (e.g. `sse`, `realtime` → `features.sse`/`features.websocket`).
- Adding a feature = create `src/modules/<feature>/` with the files above
  (`routes/` + `validation/` + `service|repository|types` as needed) and
  register it in `src/modules/index.ts`. See `FEATURE_MODULE_GUIDE.md`.
- Do NOT call Pool/Redis/RabbitMQ clients from routes. Use modules' services,
  which may call shared services in `src/services/`.
- Services default-export singleton objects with named methods.
- Repositories extend `BaseRepository`; use placeholders (`$1`) for bind
  params — never string-concatenate SQL. `pg-format` for identifier/values.
- **Database Transactions 100% WAJIB untuk SEMUA Operasi (Write & Read)**:
  - **Operasi Mutasi (`INSERT`, `UPDATE`, `DELETE`)**: **WAJIB** dieksekusi di dalam database transaction regular via transaction helper `this.withTransaction` (dari `BaseRepository`) atau `db.withTransaction(async (client) => ...)`. Dilarang mutasi tanpa transaksi.
  - **Operasi Pembacaan (`SELECT`, query, get, list, count)**: **WAJIB** dieksekusi di dalam read-only database transaction via `this.withReadOnlyTransaction` (dari `BaseRepository`) atau `db.withReadOnlyTransaction(async (client) => ...)`.
  - **Alur Read-then-Write**: Jika alur melibatkan pembacaan sebelum mutasi (validasi lalu update), seluruh alur **WAJIB** berada di dalam satu regular transaction helper yang sama.
  - **Dilarang keras melakukan manual begin/commit** (`client.query('BEGIN')` / `COMMIT` manual) atau query tanpa transaksi. Seluruh query wajib lewat helper.
  - Helper transaksi otomatis mengelola `BEGIN` / `BEGIN READ ONLY`, `COMMIT`, dan `ROLLBACK` saat terjadi error demi integritas data. Never spread a transaction across method boundaries.

## HTTP responses

Every response MUST use the envelope in `src/types/response.ts` (`ApiResponse`):

```jsonc
{ "success": bool, "message": string, "code": string, "data": unknown|null,
  "timestamp": "ISO", "requestId": "uuid" }
```

- Build responses ONLY via `ApiResponse.success|created|updated|deleted|paginated|error|...`.
- `code` MUST come from `src/constants/codes.ts` (`RESPONSE_CODES`); add new codes there, never inline.
- Paginated lists MUST use `ApiResponse.paginated` (adds `pagination` object).
- Set `res.status(...)` for non-200 (e.g. `201` for created) to match the code.
- `requestId` is auto-attached by `requestId` middleware — do not set it manually.

## Errors & Mandatory Semantic Codes

- **Mandatory Semantic Error Codes**:
  - **Dilarang keras melempar generic error tanpa semantic code** untuk kondisi domain spesifik (misal `User not found` jangan hanya menghasilkan generic `NOT_FOUND` atau `BAD_REQUEST`).
  - Selalu gunakan kelas error semantik berdedikasi atau tentukan kode eksplisit dari `src/constants/codes.ts` (`RESPONSE_CODES`):
    - `UserNotFoundError` (`USER_NOT_FOUND`)
    - `InvalidCredentialsError` (`INVALID_CREDENTIALS`)
    - `EmailAlreadyExistsError` (`EMAIL_ALREADY_EXISTS`)
    - `InvalidCurrentPasswordError` (`INVALID_CURRENT_PASSWORD`)
    - `InvalidRefreshTokenError` (`INVALID_REFRESH_TOKEN`)
    - `RefreshTokenExpiredError` (`REFRESH_TOKEN_EXPIRED`)
    - `AccessTokenExpiredError` (`ACCESS_TOKEN_EXPIRED`)
    - `InvalidAccessTokenError` (`INVALID_ACCESS_TOKEN`)
    - Atau constructor umum: `new NotFoundError('User not found', RESPONSE_CODES.USER_NOT_FOUND)`
- **Pesan Bawaan Wajib Bahasa Inggris**: Semua pesan bawaan backend wajib ditulis dalam bahasa Inggris (misal `"User not found"`, `"Invalid email or password"`). Lokalisasi bahasa lain (i18n) ditangani oleh client-side melalui mapping `errors.${apiErr.code}`.
- Wrap async handlers with `ErrorHandler.catchAsync` — do NOT write ad-hoc
  `try/catch` in handlers that swallow errors; let the error middleware respond.
- Never leak stack traces / driver errors to the client. Map to the proper
  AppError with a user/API-friendly message; log details server-side.
- Map known pg constraint violations via `src/constants/pgConstraints.ts`.

## Validation (Zod)

- All incoming data is validated at the route boundary with **Zod** through the
  `validate()` middleware (`src/utils/validator.ts`): `validate({ body, query,
  params, files })` where each value is a `z.object(...)`.
- Validated data is read from `req.validated?.<source>` — never trust raw
  `req.body/query/params` inside handlers.
- Helper types: `numeric(min?, max?)` (string→number coercion; rejects empty/
  NaN) and `boolean()` for query/body coercion. Use `z.string().email()`,
  `z.array(...)`, `z.object({...})` as usual.
- File uploads: use the `file({allowedMime?, maxBytes?, multiple?})` helper in
  the `files` source, with multer (`src/utils/upload.ts`) mounted first via
  `upload.fields([...])`. Single fields output the file or `null`; `multiple`
  outputs an array.
- Keep the Zod schemas in the module's `validation.ts`; routes call
  `validate(schema)`.
- Errors report ALL invalid fields at once: each entry is a rule
  `{ field, message, value }` under a `VALIDATION_ERROR`.

## Migrations

- Every schema change is a node-pg-migrate file under `migrations/`
  (`npm run migrate:create <name>`); no raw DDL in app code / services.
- Name them `1737..._camel-case-name.ts` following history. `up`/`down`.
- Include down migrations; keep them reversible.

## Realtime

- SSE and WebSocket must go through the realtime service/bridge
  (`src/services/realtimeService.ts`, `src/services/realtime/*`).
- Prefer the bridge over calling `sseService`/`websocketService` directly if
  both are enabled.

## Config & secrets

- `src/config/index.ts` is the single source of env config; add new env vars
  there and to `.env.example`. Never hardcode runtime config in
  services/routes.
- Never commit `.env` (gitignored), secrets, or real passwords. Use
  `.env.example` only.
- External services are toggleable via `DATABASE_ENABLED`, `REDIS_ENABLED`,
  `RABBITMQ_ENABLED`, `SSE_ENABLED`, `WS_ENABLED`, `LOG_TO_FILE`.

## Logging

- Always log through `src/utils/logger.ts` — no ad-hoc `console.log`.
- Pass structured info in context args (`{ message, ... }`); Errors are
  serialized automatically.

## Code style (enforced by eslint.config.mjs — do not fight it)

- No semicolons; single quotes (`avoidEscape: true`); 2 spaces;
  `comma-dangle: never`.
- `function name () {}` — space before `(`, `catch (`/`if (`/`} else {`.
- `object-curly-spacing` ` { a: b }`; 2-space `indent`.
- `const` (`prefer-const`), `eqeqeq` (`===`), `no-var`, template strings.
- Type-only imports use `import type { ... }` + `@typescript-eslint/consistent-type-imports`.
- Run `npm run lint` after any change; `npm run lint:fix` then re-check.

## Definition of done

1. `npm run typecheck` — clean.
2. `npm run lint` — clean (fixable via `npm run lint:fix`).
3. `npm run build` — if adding/replacing logic, `dist` builds.
4. If you changed HTTP behavior, add/adjust the request in `api-test*.http`.
5. For reversible schema upgrades, add a migration; for docs-correlated
   behavior, update the corresponding `*.md` (they live in the repo root).
6. 100% operasi database dibungkus transaction helper (Regular Transaction untuk write/update/delete, Read-Only Transaction untuk query/read).