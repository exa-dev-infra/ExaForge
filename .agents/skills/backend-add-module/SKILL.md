---
name: backend-add-module
description: >-
  Step-by-step procedure to scaffold and implement a new domain feature module in the Express TypeScript backend. Use when creating new API modules, services, or routes.
---

# Scaffolding a New Express TypeScript Module

This skill guides you through implementing a new domain feature module in the **TypeScript + Express** backend following the feature module pattern.

---

## Architecture Pattern

Every domain feature lives under `src/modules/<feature>/`:

```text
src/modules/<feature>/
├── types.ts        # Domain types, DTOs, and interface definitions
├── validation.ts   # Zod validation schemas for routes
├── repository.ts   # Database layer extending BaseRepository
├── service.ts      # Business logic singleton (default export)
└── routes.ts       # Express router with validated handlers
```

---

## Step-by-Step Implementation

### Step 1: Define Domain Types (`src/modules/<feature>/types.ts`)
```typescript
export interface <Entity> {
  id: number;
  name: string;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface Create<Entity>Dto {
  name: string;
  status?: string;
}

export interface <Entity>FilterDto {
  status?: string;
  limit?: number;
  offset?: number;
}
```

### Step 2: Define Zod Validation Schemas (`src/modules/<feature>/validation.ts`)
```typescript
import { z } from 'zod';

export const Create<Entity>Schema = z.object({
  name: z.string().min(3).max(100),
  status: z.enum(['active', 'inactive']).default('active'),
});

export const <Entity>FilterSchema = z.object({
  status: z.enum(['active', 'inactive']).optional(),
  limit: z.coerce.number().min(1).max(100).default(10),
  offset: z.coerce.number().min(0).default(0),
});
```

### Step 3: Implement Repository (`src/modules/<feature>/repository.ts`)

> **ATURAN WAJIB DATABASE TRANSACTIONS**:
> **SEMUA** operasi write/mutasi data (`create`, `update`, `delete`) **WAJIB** dibungkus dalam transaksi menggunakan `this.withTransaction(async (client) => { ... })`. Dilarang keras mengeksekusi operasi mutasi tanpa transaksi!
> Jika alur proses melibatkan pembacaan sebelum mutasi (read-then-write) atau relasi multi-tabel, seluruh operasi WAJIB menggunakan transaksi yang sama.
> Operasi read-only murni (seperti `findById`, `findAll`) **boleh langsung tanpa transaksi** via `this.query`.

```typescript
import { BaseRepository } from '@/repositories/BaseRepository';
import type { <Entity>, Create<Entity>Dto } from './types';

export class <Entity>Repository extends BaseRepository {
  async create(data: Create<Entity>Dto): Promise<<Entity>> {
    return await this.withTransaction(async (client) => {
      const query = `
        INSERT INTO <table_name> (name, status, created_at, updated_at)
        VALUES ($1, $2, NOW(), NOW())
        RETURNING id, name, status, created_at AS "createdAt", updated_at AS "updatedAt"
      `;
      const result = await client.query(query, [data.name, data.status || 'active']);
      return result.rows[0];
    });
  }

  async findById(id: number): Promise<<Entity> | null> {
    return await this.withReadOnlyTransaction(async (client) => {
      const query = `
        SELECT id, name, status, created_at AS "createdAt", updated_at AS "updatedAt"
        FROM <table_name>
        WHERE id = $1
      `;
      const result = await client.query<<Entity>>(query, [id]);
      return result.rows[0] || null;
    });
  }
}

export default new <Entity>Repository();
```

### Step 4: Implement Service (`src/modules/<feature>/service.ts`)
```typescript
import <Entity>Repository from './repository';
import type { <Entity>, Create<Entity>Dto } from './types';
import { NotFoundError } from '@/types/errors';

export class <Entity>Service {
  async create(data: Create<Entity>Dto): Promise<<Entity>> {
    return <Entity>Repository.create(data);
  }

  async getById(id: number): Promise<<Entity>> {
    const entity = await <Entity>Repository.findById(id);
    if (!entity) {
      throw new NotFoundError('<Entity> not found');
    }
    return entity;
  }
}

export default new <Entity>Service();
```

### Step 5: Implement Routes (`src/modules/<feature>/routes.ts`)
Always validate at the boundary using `validate()` and wrap async handlers with `ErrorHandler.catchAsync`:

```typescript
import { Router } from 'express';
import { validate } from '@/utils/validator';
import { ErrorHandler } from '@/middleware/errorHandler';
import { ApiResponse } from '@/types/response';
import { RESPONSE_CODES } from '@/constants/codes';
import <Entity>Service from './service';
import { Create<Entity>Schema } from './validation';

const router = Router();

router.post(
  '/',
  validate({ body: Create<Entity>Schema }),
  ErrorHandler.catchAsync(async (req, res) => {
    const data = req.validated.body;
    const result = await <Entity>Service.create(data);
    return ApiResponse.created(res, result, RESPONSE_CODES.SUCCESS);
  })
);

router.get(
  '/:id',
  ErrorHandler.catchAsync(async (req, res) => {
    const id = parseInt(req.params.id, 10);
    const result = await <Entity>Service.getById(id);
    return ApiResponse.success(res, result, RESPONSE_CODES.SUCCESS);
  })
);

export default router;
```

### Step 6: Register in `src/modules/index.ts`
Add the module to the feature registry:

```typescript
import <feature>Routes from './<feature>/routes';

export const featureModules: FeatureModule[] = [
  // ...
  {
    name: '<feature>',
    prefix: '/api/v1/<feature_plural>',
    router: <feature>Routes,
    enabled: true,
  },
];
```

---

## Quality Gate Checklist

Before completion:
1. `npm run typecheck` passes with zero errors.
2. `npm run lint` passes without complaints.
3. `npm run build` compiles `dist/` cleanly.
