# Repository Pattern Architecture

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         CLIENT                              │
│                    (HTTP Requests)                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      ROUTES LAYER                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  users.ts    │  │  auth.ts     │  │ products.ts  │      │
│  │              │  │              │  │              │      │
│  │ - Validation │  │ - JWT        │  │ - Filtering  │      │
│  │ - Error Hdl  │  │ - Auth Logic │  │ - Sorting    │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼─────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                   REPOSITORY LAYER                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          BaseRepository<T> (Abstract)                │   │
│  │  - findAll()      - delete(id)                       │   │
│  │  - findById(id)   - count()                          │   │
│  │  - findBy()       - exists()                         │   │
│  │  - findOneBy()    - withTransaction()                │   │
│  │  - create()       - query()                          │   │
│  │  - update()                                          │   │
│  └──────────────────────┬───────────────────────────────┘   │
│                         │ extends                           │
│          ┌──────────────┼──────────────┐                    │
│          │              │              │                    │
│  ┌───────▼──────┐ ┌─────▼──────┐ ┌────▼──────┐            │
│  │UserRepository│ │ProductRepo │ │YourRepo   │            │
│  │              │ │            │ │           │            │
│  │- findByEmail │ │- findBy    │ │- Custom   │            │
│  │- findByRole  │ │  Category  │ │  Methods  │            │
│  │- emailExists │ │- findBy    │ │           │            │
│  │- search()    │ │  PriceRange│ │           │            │
│  │- paginated() │ │- lowStock  │ │           │            │
│  └──────┬───────┘ └─────┬──────┘ └────┬──────┘            │
└─────────┼───────────────┼─────────────┼────────────────────┘
          │               │             │
          └───────────────┼─────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                     DATABASE LAYER                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Database (Singleton)                    │   │
│  │  - query()                                           │   │
│  │  - getClient()                                       │   │
│  │  - withTransaction()                                 │   │
│  │  - getPool()                                         │   │
│  └──────────────────────┬───────────────────────────────┘   │
└─────────────────────────┼───────────────────────────────────┘
                          │
                          ▼
                    ┌──────────┐
                    │PostgreSQL│
                    │ Database │
                    └──────────┘
```

## Request Flow Example

```
1. Client Request
   GET /users/123
   
2. Route Handler (users.ts)
   ↓
   const user = await userRepository.findById(123);
   
3. Repository Layer (UserRepository)
   ↓
   Inherits findById() from BaseRepository
   
4. BaseRepository
   ↓
   const result = await this.db.query(
     'SELECT * FROM users WHERE id = $1',
     [123]
   );
   
5. Database Layer
   ↓
   pool.query() → PostgreSQL
   
6. Response Flow (back up)
   PostgreSQL → Database → Repository → Route → Client
```

## Data Flow Visualization

```
HTTP Request
     ↓
┌────────────┐
│   Route    │  ← Validation, Business Logic
└─────┬──────┘
      ↓
┌────────────┐
│ Repository │  ← Data Access Logic
└─────┬──────┘
      ↓
┌────────────┐
│  Database  │  ← Connection Pool
└─────┬──────┘
      ↓
┌────────────┐
│ PostgreSQL │  ← Physical Database
└────────────┘
```

## Component Responsibilities

### Route Layer (users.ts, auth.ts, products.ts)

```typescript
✓ HTTP
request / response
handling
✓ Input
validation
✓ Business
logic
✓ Error
handling
✓ Authentication / Authorization
✗ NO
direct
database
queries
✗ NO
SQL
statements
```

### Repository Layer (UserRepository, ProductRepository)

```typescript
✓ Data
access
logic
✓ SQL
queries
✓ Data
transformation
✓ Custom
queries
✗ NO
business
logic
✗ NO
validation
✗ NO
HTTP
concerns
```

### Database Layer (Database singleton)

```typescript
✓ Connection
pooling
✓ Query
execution
✓ Transaction
management
✓ Connection
lifecycle
```

## Before vs After

### Before (Mixed Concerns)

```
┌─────────────┐
│   Route     │
│             │
│ - Validate  │
│ - SQL Query │  ← Mixed!
│ - Transform │
│ - Response  │
└─────────────┘
```

### After (Separated Concerns)

```
┌─────────────┐
│   Route     │
│ - Validate  │
│ - Business  │
└──────┬──────┘
       │
┌──────▼──────┐
│ Repository  │
│ - SQL Query │
│ - Transform │
└─────────────┘
```

## File Organization

```
src/
├── routes/                    ← HTTP Layer
│   ├── users.ts              (Uses UserRepository)
│   ├── auth.ts               (Uses UserRepository)
│   └── products.ts           (Uses ProductRepository)
│
├── repositories/              ← Data Access Layer
│   ├── BaseRepository.ts     (Abstract base class)
│   ├── UserRepository.ts     (User-specific operations)
│   ├── ProductRepository.ts  (Product-specific operations)
│   └── index.ts              (Export hub)
│
├── config/                    ← Infrastructure Layer
│   ├── database.ts           (Connection pool)
│   ├── redis.ts
│   └── rabbitmq.ts
│
├── middleware/
├── services/
└── types/
```

## Dependency Flow

```
Route
  ↓ depends on
Repository
  ↓ depends on
Database
  ↓ depends on
PostgreSQL
```

**Important:** Dependencies flow downward only!

- Routes don't know about Database
- Routes only know about Repositories
- Repositories don't know about Routes

## Testing Architecture

```
Unit Tests (Isolated)
├── Route Tests
│   └── Mock Repository ✓
│
├── Repository Tests
│   └── Mock Database ✓
│
└── Database Tests
    └── Test Database ✓

Integration Tests (Connected)
└── Route → Repository → Database → PostgreSQL ✓
```

## Pattern Benefits Visualization

```
WITHOUT Repository Pattern:
┌────────┐     ┌────────┐     ┌────────┐
│Route 1 │────→│   DB   │     │Route 2 │
└────────┘     └────────┘     └───┬────┘
                                   │
                                   ↓
                              ┌────────┐
                              │   DB   │
                              └────────┘
Problem: Duplicate queries, inconsistent logic


WITH Repository Pattern:
┌────────┐     ┌────────┐     ┌────────┐
│Route 1 │────→│  Repo  │←────│Route 2 │
└────────┘     └───┬────┘     └────────┘
                   │
                   ↓
              ┌────────┐
              │   DB   │
              └────────┘
Solution: Centralized, reusable, consistent
```

## Summary

```
┌─────────────────────────────────────────┐
│  ✓ Clean Separation of Concerns        │
│  ✓ Single Responsibility Principle     │
│  ✓ DRY (Don't Repeat Yourself)         │
│  ✓ Easy to Test                        │
│  ✓ Easy to Maintain                    │
│  ✓ Type-Safe with TypeScript           │
│  ✓ Scalable Architecture               │
└─────────────────────────────────────────┘
```

---

This architecture follows industry best practices and makes your code:

- **Maintainable** - Easy to understand and modify
- **Testable** - Each layer can be tested independently
- **Scalable** - Easy to add new entities
- **Professional** - Standard enterprise pattern
