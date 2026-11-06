# Repository Pattern - Quick Reference

## Installation Complete ✅

The Repository Pattern has been successfully implemented in your TypeScript/Express application.

## What Was Created

### 1. Base Repository (`src/repositories/BaseRepository.ts`)

Abstract base class providing common CRUD operations:

- `findAll()` - Get all records
- `findById(id)` - Find by ID
- `findBy(field, value)` - Find by any field
- `findOneBy(field, value)` - Find one by any field
- `create(data)` - Create (abstract - must implement)
- `update(id, data)` - Update (abstract - must implement)
- `delete(id)` - Delete record
- `count()` - Count records
- `exists(id)` - Check existence
- `withTransaction(fn)` - Transaction support

### 2. User Repository (`src/repositories/UserRepository.ts`)

Full implementation for User entity:

- Standard CRUD operations
- `findByEmail(email)` - Find user by email
- `findByRole(role)` - Find users by role
- `emailExists(email)` - Check if email exists
- `findAllPaginated(page, limit)` - Paginated results
- `search(searchTerm)` - Search by name/email

### 3. Updated Routes

- ✅ `src/routes/users.ts` - Updated to use UserRepository
- ✅ `src/routes/auth.ts` - Updated to use UserRepository

### 4. Documentation

- 📚 `REPOSITORY_PATTERN_GUIDE.md` - Complete guide

## Quick Usage

### Import Repository

```typescript
import userRepository from '../repositories/UserRepository';
```

### Basic Operations

```typescript
// Get all
const users = await userRepository.findAll();

// Get by ID
const user = await userRepository.findById(1);

// Create
const newUser = await userRepository.create({
    name: 'John Doe',
    email: 'john@example.com'
});

// Update
const updated = await userRepository.update(1, {
    name: 'Jane Doe'
});

// Delete
const deleted = await userRepository.delete(1);
```

### Custom Methods

```typescript
// Find by email
const user = await userRepository.findByEmail('john@example.com');

// Check if email exists
const exists = await userRepository.emailExists('john@example.com');

// Paginated results
const result = await userRepository.findAllPaginated(1, 10);
// Returns: { data, total, page, limit, totalPages }

// Search
const users = await userRepository.search('john');
```

## Creating a New Repository

### 1. Create the file

```typescript
// src/repositories/ProductRepository.ts
import {BaseRepository} from './BaseRepository';

export interface Product {
    id: number;
    name: string;
    price: number;
    stock: number;
}

class ProductRepository extends BaseRepository<Product> {
    constructor() {
        super('products'); // table name
    }

    async create(data: Partial<Product>): Promise<Product> {
        const result = await this.query(
            `INSERT INTO products (name, price, stock) 
             VALUES ($1, $2, $3) RETURNING *`,
            [data.name, data.price, data.stock]
        );
        return result.rows[0];
    }

    async update(id: string | number, data: Partial<Product>): Promise<Product | null> {
        // Implement update logic
        // See UserRepository.ts for example
    }

    // Add custom methods
    async findLowStock(threshold: number = 10): Promise<Product[]> {
        const result = await this.query(
            `SELECT * FROM products WHERE stock < $1`,
            [threshold]
        );
        return result.rows;
    }
}

export default new ProductRepository();
```

### 2. Use in routes

```typescript
import productRepository from '../repositories/ProductRepository';

router.get('/products', async (req, res) => {
    const products = await productRepository.findAll();
    res.json(ApiResponse.success(products));
});
```

## Updated Route Examples

### Before (Direct DB)

```typescript
const db = Database.getInstance();
const result = await db.query('SELECT * FROM users WHERE id = $1', [id]);
if (result.rowCount === 0) {
    res.status(404).json({error: 'Not found'});
    return;
}
const user = result.rows[0];
```

### After (Repository Pattern)

```typescript
const user = await userRepository.findById(id);
if (!user) {
    throw new NotFoundError('User not found');
}
```

## Benefits

✅ **Cleaner Code**: Less boilerplate, more readable  
✅ **Reusability**: Common operations defined once  
✅ **Type Safety**: Full TypeScript support  
✅ **Testability**: Easy to mock in tests  
✅ **Maintainability**: Centralized data access logic  
✅ **Consistency**: Standardized patterns across app

## Testing

```typescript
import userRepository from '../repositories/UserRepository';

jest.mock('../repositories/UserRepository');

test('should find user', async () => {
    (userRepository.findById as jest.Mock).mockResolvedValue({
        id: 1,
        name: 'John'
    });

    const user = await userRepository.findById(1);
    expect(user.name).toBe('John');
});
```

## Transaction Example

```typescript
import Database from '../config/database';
import userRepository from '../repositories/UserRepository';

const db = Database.getInstance();

await db.withTransaction(async (client) => {
    const user = await userRepository.create({
        name: 'John',
        email: 'john@example.com'
    });

    // More operations...
    // If any fail, all are rolled back
});
```

## File Structure

```
src/
├── repositories/
│   ├── BaseRepository.ts       # Base class with common operations
│   ├── UserRepository.ts       # User entity repository
│   └── [YourEntity]Repository.ts  # Add more as needed
├── routes/
│   ├── users.ts               # Updated to use repository
│   └── auth.ts                # Updated to use repository
└── types/
    └── ...
```

## Next Steps

1. ✅ Repository pattern is now active in your application
2. 📖 Read `REPOSITORY_PATTERN_GUIDE.md` for detailed examples
3. 🔨 Create new repositories for other entities (products, orders, etc.)
4. 🧪 Add tests for your repositories
5. 🚀 Enjoy cleaner, more maintainable code!

## Common Patterns

### With Error Handling

```typescript
router.get('/:id', ErrorHandler.catchAsync(async (req, res) => {
    const user = await userRepository.findById(req.params.id);

    if (!user) {
        throw new NotFoundError('User not found');
    }

    res.json(ApiResponse.success(user));
}));
```

### With Validation

```typescript
router.post('/', ErrorHandler.catchAsync(async (req, res) => {
    const {email} = req.body;

    if (await userRepository.emailExists(email)) {
        throw new ValidationError('Email already exists');
    }

    const user = await userRepository.create(req.body);
    res.status(201).json(ApiResponse.created(user));
}));
```

### With Pagination

```typescript
router.get('/', async (req, res) => {
    const page = parseInt(req.query.page) || 1;
    const limit = parseInt(req.query.limit) || 10;

    const result = await userRepository.findAllPaginated(page, limit);

    res.json({
        ...ApiResponse.success(result.data),
        pagination: {
            page: result.page,
            limit: result.limit,
            total: result.total,
            totalPages: result.totalPages
        }
    });
});
```

## Documentation

- 📚 [Complete Repository Pattern Guide](REPOSITORY_PATTERN_GUIDE.md)
- 📚 [Error Handling Guide](ERROR_HANDLING.md)
- 📚 [Transaction Guide](TRANSACTION_GUIDE.md)
- 📚 [Quick Reference](QUICK_REFERENCE.md)

---

**Repository Pattern Implementation Complete!** 🎉

Your application now uses a clean, maintainable repository pattern for all data access operations.
