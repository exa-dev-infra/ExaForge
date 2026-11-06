# FEATURE_MODULE_GUIDE.md

How to add a new feature to this backend. Every feature is a self-contained
folder under `src/modules/<feature>/` — copy the pattern below and register the
router in `src/modules/index.ts`.

## Module anatomy

One feature = one folder, flat files:

```
src/modules/<feature>/
├── routes.ts        Express router (controllers inline), default exports a Router
├── service.ts       business logic; default-exports a singleton object
├── repository.ts    ONLY if DB-backed; class extends BaseRepository<T>
├── types.ts         domain types / DTOs
└── validation.ts    Zod schemas consumed by validate()
```

Shared infra stays OUT of modules:
- `src/services/` — jwt, password, cache, queue, realtime, sse, websocket
- `src/repositories/BaseRepository.ts` — base for every repository
- `src/config/`, `src/middleware/`, `src/types/`, `src/utils/`, `src/constants/`

## Step-by-step: add a `categories` feature

### 1. Migration (table)

```bash
npm run migrate:create categories
```

In the generated file: `up` creates the table, `down` drops it. Keep the
timestamped `17..._camel-case-name.ts` naming.

### 2. `src/modules/categories/types.ts`

```ts
export interface Category {
  id: number
  name: string
  created_at?: Date
}

export interface CreateCategoryDTO {
  name: string
}
```

### 3. `src/modules/categories/repository.ts`

```ts
import { BaseRepository } from '../../repositories/BaseRepository'
import type { Category, CreateCategoryDTO } from './types'

class CategoryRepository extends BaseRepository<Category> {
  constructor () { super('categories') }

  async create (data: CreateCategoryDTO): Promise<Category> {
    const result = await this.query(
      `INSERT INTO ${this.tableName} (name, created_at) VALUES ($1, NOW()) RETURNING *`,
      [data.name]
    )
    return result.rows[0] as Category
  }

  async update (id: string | number, data: Partial<CreateCategoryDTO>): Promise<Category | null> {
    const result = await this.query(
      `UPDATE ${this.tableName}
            SET name = $1, updated_at = NOW()
            WHERE id = $2 RETURNING *`,
      [data.name, id]
    )
    return (result.rowCount ?? 0) > 0 ? (result.rows[0] as Category) : null
  }
}

export default new CategoryRepository()
```

Use `$1` placeholders — never string-concatenate values. `this.tableName`,
`this.query`, `this.findById`, `this.findBy`, `findOneBy`, `exists`, `delete`,
`count` come from `BaseRepository`.

### 4. `src/modules/categories/service.ts`

```ts
import type { Category, CreateCategoryDTO } from './types'
import categoryRepository from './repository'
import { NotFoundError } from '../../types/errors'

class CategoryService {
  async list (): Promise<Category[]> { return await categoryRepository.findAll() }

  async getById (id: string | number): Promise<Category> {
    const category = await categoryRepository.findById(id)
    if (!category) { throw new NotFoundError('Category not found') }
    return category
  }

  async create (data: CreateCategoryDTO): Promise<Category> {
    return await categoryRepository.create(data)
  }

  async remove (id: string | number): Promise<void> {
    const deleted = await categoryRepository.delete(id)
    if (!deleted) { throw new NotFoundError('Category not found') }
  }
}

export default new CategoryService()
```

### 5. `src/modules/categories/validation.ts`

```ts
import { z } from 'zod'

export const categoryIdSchema = {
  params: z.object({ id: z.string().min(1) })
}

export const createCategorySchema = {
  body: z.object({
    name: z.string().min(2).max(100)
  })
}
```

Use `zod` directly. For coercing numbers/booleans from query strings use the
helpers from `../../utils/validator`:

```ts
import { numeric, bool } from '../../utils/validator'
// numeric(min?, max?)  -> number, rejects empty/NaN ('5' → 5)
// bool()               -> 'true'/'1'/'yes'/'on' → true, 'false'/'0'/'no' → false
```

### 6. `src/modules/categories/routes.ts`

```ts
import type { Request, Response } from 'express'
import { Router } from 'express'
import { ApiResponse } from '../../types/response'
import ErrorHandler from '../../middleware/errorHandler'
import { validate } from '../../utils/validator'
import categoryService from './service'
import { categoryIdSchema, createCategorySchema } from './validation'

const router = Router()

router.get('/', ErrorHandler.catchAsync(async (_req: Request, res: Response) => {
  const categories = await categoryService.list()
  res.json(ApiResponse.success(categories, 'Categories retrieved successfully'))
}))

router.get('/:id', validate(categoryIdSchema), ErrorHandler.catchAsync(async (req: Request, res: Response) => {
  const category = await categoryService.getById(req.params.id)
  res.json(ApiResponse.success(category, 'Category retrieved successfully'))
}))

router.post('/', validate(createCategorySchema), ErrorHandler.catchAsync(async (req: Request, res: Response) => {
  const category = await categoryService.create(
    req.validated?.body as unknown as Parameters<typeof categoryService.create>[0]
  )
  res.status(201).json(ApiResponse.created(category, 'Category created successfully'))
}))

router.delete('/:id', validate(categoryIdSchema), ErrorHandler.catchAsync(async (req: Request, res: Response) => {
  await categoryService.remove(req.params.id)
  res.json(ApiResponse.deleted('Category deleted successfully'))
}))

export default router
```

Note `ErrorHandler.catchAsync` on every async handler, `requestId` is attached
automatically, and `req.validated?.<source>` — never `req.body` directly.

### 6. Register in `src/modules/index.ts`

```ts
import categoriesRoutes from './categories/routes'

// add to featureModules[]:
{ name: 'categories', mountPath: '/categories', enabled: true, router: categoriesRoutes }
```

`mountModules(app)` in `app.ts` mounts every enabled module in order.
Flag-gated modules set `enabled` from `features` (see `sse`/`realtime`).

### 7. Verify

```bash
npm run typecheck && npm run lint && npm run migrate:up && npm run dev
```

## Rule checklist

- [ ] Zod schemas live in `validation.ts`; every route calls `validate(...)`.
- [ ] Handler reads `req.validated?.<source>` only.
- [ ] Routes never touch Pool/Redis/RabbitMQ — go through `service.ts`.
- [ ] Service throws `AppError` subclasses; wrapped with `ErrorHandler.catchAsync`.
- [ ] Responses built with `ApiResponse.*` + codes from `RESPONSE_CODES`.
- [ ] Repositories extend `BaseRepository`, parameters via `$1` placeholders.
- [ ] Schema changes go through migrations only.
- [ ] `typecheck`, `lint`, `build` clean; `.http` request added if new HTTP behavior.