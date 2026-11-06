# Repository Pattern Guide

## Overview

The Repository Pattern is implemented in this project to separate the data access logic from the business logic. This
provides a cleaner architecture, better testability, and easier maintenance.

## Structure

```
src/
├── repositories/
│   ├── BaseRepository.ts      # Abstract base class with common CRUD operations
│   └── UserRepository.ts      # User-specific repository implementation
```

## Components

### 1. BaseRepository (Abstract Class)

The `BaseRepository` provides common database operations that all repositories inherit:

**Core Methods:**

- `findAll()` - Get all records
- `findById(id)` - Find a single record by ID
- `findBy(field, value)` - Find records by any field
- `findOneBy(field, value)` - Find one record by any field
- `create(data)` - Create a new record (abstract - must be implemented)
- `update(id, data)` - Update a record (abstract - must be implemented)
- `delete(id)` - Delete a record
- `count()` - Count total records
- `exists(id)` - Check if a record exists
- `withTransaction(fn)` - Execute operations within a transaction

**Protected Methods:**

- `query(text, params)` - Execute custom SQL queries

### 2. UserRepository

Extends `BaseRepository` with user-specific operations:

**Standard Operations:**

- `create(data)` - Create a new user
- `update(id, data)` - Update user information
- `findAll()` - Get all users (inherited)
- `findById(id)` - Find user by ID (inherited)
- `delete(id)` - Delete a user (inherited)

**Custom Operations:**

- `findByEmail(email)` - Find user by email
- `findByRole(role)` - Find users by role
- `emailExists(email)` - Check if email exists
- `findAllPaginated(page, limit)` - Get paginated users
- `search(searchTerm)` - Search users by name or email

## Usage Examples

### Basic CRUD Operations

```typescript
import userRepository from '../repositories/UserRepository';

// CREATE
const newUser = await userRepository.create({
    name: 'John Doe',
    email: 'john@example.com',
    password: 'hashedPassword',
    role: 'user'
});

// READ - All users
const users = await userRepository.findAll();

// READ - By ID
const user = await userRepository.findById(1);

// READ - By email
const user = await userRepository.findByEmail('john@example.com');

// UPDATE
const updatedUser = await userRepository.update(1, {
    name: 'John Smith'
});

// DELETE
const deleted = await userRepository.delete(1);
```

### In Route Handlers

```typescript
// src/routes/users.ts
import userRepository from '../repositories/UserRepository';
import {NotFoundError, ValidationError} from '../types/errors';

// GET all users
router.get('/', ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const users = await userRepository.findAll();
    const response = ApiResponse.success(users, 'Users retrieved successfully');
    res.json(response);
}));

// GET user by ID with error handling
router.get('/:id', ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const user = await userRepository.findById(req.params.id);
    
    if (!user) {
        throw new NotFoundError('User not found');
    }
    
    const response = ApiResponse.success(user, 'User retrieved successfully');
    res.json(response);
}));

// CREATE with validation
router.post('/', ErrorHandler.catchAsync(async (req: Request, res: Response) => {
    const {name, email} = req.body;
    
    if (!name || !email) {
        throw new ValidationError('Name and email are required');
    }
    
    // Check for duplicate email
    const existingUser = await userRepository.findByEmail(email);
    if (existingUser) {
        throw new ValidationError('Email already exists');
    }
    
    const user = await userRepository.create({name, email});
    const response = ApiResponse.created(user, 'User created successfully');
    res.status(201).json(response);
}));
```

### Advanced Usage

#### Pagination

```typescript
router.get('/users/paginated', async (req: Request, res: Response) => {
    const page = parseInt(req.query.page as string) || 1;
    const limit = parseInt(req.query.limit as string) || 10;
    
    const result = await userRepository.findAllPaginated(page, limit);
    
    res.json({
        success: true,
        data: result.data,
        pagination: {
            page: result.page,
            limit: result.limit,
            total: result.total,
            totalPages: result.totalPages
        }
    });
});
```

#### Search

```typescript
router.get('/users/search', async (req: Request, res: Response) => {
    const searchTerm = req.query.q as string;
    
    if (!searchTerm) {
        throw new ValidationError('Search term is required');
    }
    
    const users = await userRepository.search(searchTerm);
    res.json(ApiResponse.success(users, 'Search results'));
});
```

#### Custom Queries

```typescript
// In your repository class
async findActiveUsers(): Promise<User[]> {
    const result = await this.query(
        `SELECT * FROM ${this.tableName} 
         WHERE is_active = true 
         ORDER BY created_at DESC`
    );
    return result.rows as User[];
}
```

## Creating a New Repository

### Step 1: Define the Entity Interface

```typescript
// src/repositories/ProductRepository.ts
export interface Product {
    id: number;
    name: string;
    description: string;
    price: number;
    stock: number;
    created_at?: Date;
    updated_at?: Date;
}

export interface CreateProductDTO {
    name: string;
    description: string;
    price: number;
    stock: number;
}

export interface UpdateProductDTO {
    name?: string;
    description?: string;
    price?: number;
    stock?: number;
}
```

### Step 2: Create the Repository Class

```typescript
import {BaseRepository} from './BaseRepository';

class ProductRepository extends BaseRepository<Product> {
    constructor() {
        super('products');
    }

    async create(data: CreateProductDTO): Promise<Product> {
        const {name, description, price, stock} = data;
        
        const result = await this.query(
            `INSERT INTO ${this.tableName} (name, description, price, stock, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, NOW(), NOW()) 
             RETURNING *`,
            [name, description, price, stock]
        );
        
        return result.rows[0] as Product;
    }

    async update(id: string | number, data: UpdateProductDTO): Promise<Product | null> {
        const fields: string[] = [];
        const values: any[] = [];
        let paramIndex = 1;

        if (data.name !== undefined) {
            fields.push(`name = $${paramIndex++}`);
            values.push(data.name);
        }

        if (data.description !== undefined) {
            fields.push(`description = $${paramIndex++}`);
            values.push(data.description);
        }

        if (data.price !== undefined) {
            fields.push(`price = $${paramIndex++}`);
            values.push(data.price);
        }

        if (data.stock !== undefined) {
            fields.push(`stock = $${paramIndex++}`);
            values.push(data.stock);
        }

        if (fields.length === 0) {
            return await this.findById(id);
        }

        fields.push(`updated_at = NOW()`);
        values.push(id);

        const result = await this.query(
            `UPDATE ${this.tableName} 
             SET ${fields.join(', ')} 
             WHERE id = $${paramIndex} 
             RETURNING *`,
            values
        );

        return (result.rowCount ?? 0) > 0 ? (result.rows[0] as Product) : null;
    }

    // Custom methods
    async findByPriceRange(minPrice: number, maxPrice: number): Promise<Product[]> {
        const result = await this.query(
            `SELECT * FROM ${this.tableName} 
             WHERE price BETWEEN $1 AND $2 
             ORDER BY price ASC`,
            [minPrice, maxPrice]
        );
        return result.rows as Product[];
    }

    async findLowStock(threshold: number = 10): Promise<Product[]> {
        const result = await this.query(
            `SELECT * FROM ${this.tableName} 
             WHERE stock < $1 
             ORDER BY stock ASC`,
            [threshold]
        );
        return result.rows as Product[];
    }
}

export default new ProductRepository();
```

### Step 3: Use in Routes

```typescript
import productRepository from '../repositories/ProductRepository';

router.get('/products', async (req: Request, res: Response) => {
    const products = await productRepository.findAll();
    res.json(ApiResponse.success(products));
});

router.get('/products/low-stock', async (req: Request, res: Response) => {
    const products = await productRepository.findLowStock(10);
    res.json(ApiResponse.success(products));
});
```

## Transaction Support

The repository pattern supports database transactions through the `withTransaction` method:

```typescript
import Database from '../config/database';
import userRepository from '../repositories/UserRepository';
import orderRepository from '../repositories/OrderRepository';

const db = Database.getInstance();

await db.withTransaction(async (client) => {
    // Create user
    const user = await userRepository.create({
        name: 'John Doe',
        email: 'john@example.com'
    });
    
    // Create order for user
    const order = await orderRepository.create({
        userId: user.id,
        total: 100
    });
    
    // If any operation fails, all changes are rolled back
});
```

## Benefits

1. **Separation of Concerns**: Data access logic is separated from business logic
2. **Reusability**: Common database operations are defined once in BaseRepository
3. **Testability**: Easy to mock repositories in unit tests
4. **Maintainability**: Changes to database queries are centralized
5. **Type Safety**: Full TypeScript support with interfaces
6. **Consistency**: Standardized way to access data across the application

## Best Practices

1. **Keep repositories focused**: Each repository should handle one entity
2. **Use DTOs**: Define separate interfaces for create/update operations
3. **Handle errors at the route level**: Let errors bubble up from repositories
4. **Add custom methods**: Extend repositories with domain-specific queries
5. **Use transactions**: Wrap multiple operations in transactions when needed
6. **Document complex queries**: Add comments for non-obvious SQL queries
7. **Validate at the route level**: Keep repositories focused on data access

## Migration from Direct Database Calls

### Before (Direct DB calls)

```typescript
// Old way
const db = Database.getInstance();
const result = await db.query('SELECT * FROM users WHERE id = $1', [id]);
const user = result.rows[0];
```

### After (Repository Pattern)

```typescript
// New way
const user = await userRepository.findById(id);
```

## Testing

Repositories are easy to mock in tests:

```typescript
// In your test file
import userRepository from '../repositories/UserRepository';

jest.mock('../repositories/UserRepository');

describe('User Routes', () => {
    it('should get user by id', async () => {
        // Mock the repository method
        (userRepository.findById as jest.Mock).mockResolvedValue({
            id: 1,
            name: 'John Doe',
            email: 'john@example.com'
        });
        
        // Test your route
        const response = await request(app).get('/users/1');
        
        expect(response.status).toBe(200);
        expect(response.body.data.name).toBe('John Doe');
    });
});
```

## Related Documentation

- [Error Handling Guide](ERROR_HANDLING.md)
- [Transaction Guide](TRANSACTION_GUIDE.md)
- [API Response Format](QUICK_REFERENCE.md)

## Conclusion

The Repository Pattern provides a clean, maintainable, and testable way to manage data access in your application. By
following this guide, you can create consistent and reliable data access layers for all your entities.
