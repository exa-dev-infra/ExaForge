# Service Layer Pattern - Complete Guide

## 🎯 Overview

Service Layer adalah lapisan tambahan yang memisahkan **business logic** dari routes. Dengan pola ini, routes hanya
menangani HTTP request/response, sementara semua validasi, transformasi data, dan logika bisnis berada di service layer.

## 📐 Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    CLIENT (HTTP)                        │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                  ROUTES LAYER                           │
│  - Parse request                                        │
│  - Call service methods                                 │
│  - Format response                                      │
│  - Handle HTTP concerns                                 │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                 SERVICE LAYER                           │
│  - Business logic                                       │
│  - Validation                                           │
│  - Data transformation                                  │
│  - Orchestrate multiple repositories                    │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│               REPOSITORY LAYER                          │
│  - Data access                                          │
│  - Database queries                                     │
│  - CRUD operations                                      │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
                  ┌──────────┐
                  │ Database │
                  └──────────┘
```

## 📁 Project Structure

```
src/
├── routes/
│   ├── users.ts              ← HTTP layer (thin)
│   ├── auth.ts               ← HTTP layer (thin)
│   └── products.ts           ← HTTP layer (thin)
│
├── services/                 ← NEW! Business Logic
│   ├── userService.ts        ← User business logic
│   ├── authService.ts        ← Auth business logic
│   └── productService.ts     ← Product business logic
│
└── repositories/
    ├── UserRepository.ts     ← Data access
    └── ProductRepository.ts  ← Data access
```

## ✅ Benefits

### Before (Logic in Routes)

```typescript
// ❌ Routes terlalu besar, logic tercampur
router.post('/', async (req, res) => {
    const {name, email} = req.body;

    // Validation di route
    if (!name || !email) {
        throw new ValidationError('Required');
    }

    // Email validation di route
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
        throw new ValidationError('Invalid email');
    }

    // Check duplicate di route
    const existing = await userRepository.findByEmail(email);
    if (existing) {
        throw new ValidationError('Email exists');
    }

    // Create user
    const user = await userRepository.create({name, email});

    res.json(ApiResponse.created(user));
});
```

### After (With Service Layer)

```typescript
// ✅ Routes bersih, hanya HTTP handling
router.post('/', async (req, res) => {
    const user = await userService.createUser(req.body);
    res.status(201).json(ApiResponse.created(user));
});

// ✅ Logic ada di service
class UserService {
    async createUser(data) {
        // All validation here
        this.validateUserData(data);

        // Business logic here
        const user = await userRepository.create(data);
        return user;
    }
}
```

## 📊 Comparison

| Aspect              | Without Service | With Service     |
|---------------------|-----------------|------------------|
| **Route Size**      | 50-100 lines    | 5-10 lines       |
| **Testability**     | Hard to test    | Easy to mock     |
| **Reusability**     | Can't reuse     | Reusable         |
| **Maintainability** | Complex         | Simple           |
| **Separation**      | Mixed concerns  | Clean separation |

---

## 🚀 Implementation Examples

### 1. UserService

**Location:** `src/services/userService.ts`

```typescript
class UserService {
    // Simple CRUD with validation
    async createUser(data: CreateUserDTO): Promise<User> {
        // Validate
        if (!data.name || !data.email) {
            throw new ValidationError('Name and email required');
        }

        // Check duplicate
        const exists = await userRepository.findByEmail(data.email);
        if (exists) {
            throw new ValidationError('Email exists');
        }

        // Create
        return await userRepository.create(data);
    }

    async getUserById(id: string | number): Promise<User> {
        const user = await userRepository.findById(id);

        if (!user) {
            throw new NotFoundError('User not found');
        }

        return user;
    }
}
```

### 2. AuthService

**Location:** `src/services/authService.ts`

```typescript
class AuthService {
    // Complex business logic with multiple steps
    async register(data: RegisterDTO): Promise<AuthResponse> {
        // 1. Validate
        this.validateRegistrationData(data);

        // 2. Check duplicate
        const exists = await userRepository.findByEmail(data.email);
        if (exists) throw new ValidationError('Email exists');

        // 3. Hash password
        const hashedPassword = await passwordService.hash(data.password);

        // 4. Create user
        const user = await userRepository.create({
            ...data,
            password: hashedPassword
        });

        // 5. Generate tokens
        const tokens = jwtService.generateTokenPair({
            userId: user.id,
            email: user.email
        });

        // 6. Store refresh token
        await redis.set(`refresh_token:${user.id}`, tokens.refreshToken);

        // 7. Return response
        return {user, tokens};
    }

    async login(data: LoginDTO): Promise<AuthResponse> {
        // Multi-step process
        const user = await this.validateCredentials(data);
        const tokens = await this.generateTokens(user);
        await this.updateLastLogin(user.id);

        return {user, tokens};
    }
}
```

### 3. ProductService

**Location:** `src/services/productService.ts`

```typescript
class ProductService {
    async createProduct(data: CreateProductDTO): Promise<Product> {
        // Validate business rules
        if (data.price < 0) {
            throw new ValidationError('Price cannot be negative');
        }

        if (data.stock < 0) {
            throw new ValidationError('Stock cannot be negative');
        }

        return await productRepository.create(data);
    }

    async updateProductStock(id: string, quantity: number): Promise<Product> {
        // Get current state
        const product = await productRepository.findById(id);
        if (!product) throw new NotFoundError('Product not found');

        // Business logic: check if stock would go negative
        if (product.stock + quantity < 0) {
            throw new ValidationError('Insufficient stock', {
                current: product.stock,
                requested: Math.abs(quantity)
            });
        }

        // Update
        return await productRepository.updateStock(id, quantity);
    }
}
```

---

## 🔄 Updated Routes (Thin Controllers)

### Before vs After

#### Before (Fat Controller)

```typescript
// users.ts - 104 lines
router.post('/', async (req, res) => {
    const {name, email} = req.body;

    // 20 lines of validation
    if (!name || !email) { ...
    }
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) { ...
    }

    // 10 lines of business logic
    const existing = await userRepository.findByEmail(email);
    if (existing) { ...
    }

    // 5 lines of creation
    const user = await userRepository.create({name, email});

    res.status(201).json(ApiResponse.created(user));
});
```

#### After (Thin Controller)

```typescript
// users.ts - 56 lines (48% reduction!)
router.post('/', ErrorHandler.catchAsync(async (req, res) => {
    const user = await userService.createUser(req.body);
    res.status(201).json(ApiResponse.created(user));
}));
```

### Complete Updated Routes

#### users.ts (Clean!)

```typescript
import userService from '../services/userService';

// GET all users
router.get('/', async (req, res) => {
    const users = await userService.getAllUsers();
    res.json(ApiResponse.success(users));
});

// GET user by ID
router.get('/:id', async (req, res) => {
    const user = await userService.getUserById(req.params.id);
    res.json(ApiResponse.success(user));
});

// CREATE user
router.post('/', async (req, res) => {
    const user = await userService.createUser(req.body);
    res.status(201).json(ApiResponse.created(user));
});

// UPDATE user
router.put('/:id', async (req, res) => {
    const user = await userService.updateUser(req.params.id, req.body);
    res.json(ApiResponse.success(user));
});

// DELETE user
router.delete('/:id', async (req, res) => {
    await userService.deleteUser(req.params.id);
    res.json(ApiResponse.deleted('User deleted'));
});
```

#### auth.ts (Super Clean!)

```typescript
import authService from '../services/authService';

// REGISTER
router.post('/register', async (req, res) => {
    const result = await authService.register(req.body);
    res.status(201).json(ApiResponse.created(result));
});

// LOGIN
router.post('/login', async (req, res) => {
    const result = await authService.login(req.body);
    res.json(ApiResponse.success(result));
});

// REFRESH TOKEN
router.post('/refresh', async (req, res) => {
    const result = await authService.refreshToken(req.body.refreshToken);
    res.json(ApiResponse.success(result));
});

// LOGOUT
router.post('/logout', async (req, res) => {
    await authService.logout(req.body.userId);
    res.json(ApiResponse.success(null, 'Logged out'));
});

// CHANGE PASSWORD
router.post('/change-password', async (req, res) => {
    const userId = (req as any).user?.userId;
    await authService.changePassword(userId, req.body.currentPassword, req.body.newPassword);
    res.json(ApiResponse.success(null, 'Password changed'));
});

// GET PROFILE
router.get('/me', async (req, res) => {
    const userId = (req as any).user?.userId;
    const user = await authService.getProfile(userId);
    res.json(ApiResponse.success(user));
});
```

---

## 💡 When to Use Service Layer

### ✅ Use Service When:

1. **Complex Validation**
   ```typescript
   // Multiple validation rules
   async createProduct(data) {
       this.validatePrice(data.price);
       this.validateStock(data.stock);
       this.validateCategory(data.category);
       // ...
   }
   ```

2. **Multiple Repository Calls**
   ```typescript
   // Orchestrating multiple data sources
   async createOrder(data) {
       const user = await userRepository.findById(data.userId);
       const product = await productRepository.findById(data.productId);
       const order = await orderRepository.create({...});
       await productRepository.updateStock(product.id, -data.quantity);
       return order;
   }
   ```

3. **Business Logic**
   ```typescript
   // Business rules
   async applyDiscount(orderId, discountCode) {
       const order = await orderRepository.findById(orderId);
       const discount = await this.validateDiscountCode(discountCode);
       const newTotal = this.calculateDiscountedPrice(order.total, discount);
       return await orderRepository.update(orderId, {total: newTotal});
   }
   ```

4. **External Service Integration**
   ```typescript
   // Calling external APIs
   async sendWelcomeEmail(userId) {
       const user = await userRepository.findById(userId);
       await emailService.send(user.email, 'welcome-template');
       await queueService.publish('user-registered', {userId: user.id});
   }
   ```

### ❌ Don't Need Service When:

Simple CRUD that just passes through to repository:

```typescript
// This is OK in route directly
router.get('/', async (req, res) => {
    const items = await repository.findAll();
    res.json(items);
});
```

---

## 🧪 Testing Benefits

### Before (Hard to Test)

```typescript
// Have to mock HTTP request/response
test('create user', async () => {
    const req = {body: {name: 'John', email: 'john@test.com'}};
    const res = {
        status: jest.fn().mockReturnThis(),
        json: jest.fn()
    };

    await userRoute(req, res); // Complex!
});
```

### After (Easy to Test)

```typescript
// Just test service methods
test('create user', async () => {
    const user = await userService.createUser({
        name: 'John',
        email: 'john@test.com'
    });

    expect(user.name).toBe('John');
    expect(user.email).toBe('john@test.com');
});
```

---

## 📝 Best Practices

### 1. Keep Services Focused

```typescript
// ✅ Good: One service per domain
class UserService {
...
}

class AuthService {
...
}

class ProductService {
...
}

// ❌ Bad: Mixed responsibilities
class UserAuthProductService {
...
}
```

### 2. Services Can Call Other Services

```typescript
class OrderService {
    async createOrder(data) {
        // OK to call other services
        const user = await userService.getUserById(data.userId);
        const product = await productService.getProductById(data.productId);

        // Validate business rules
        if (product.stock < data.quantity) {
            throw new Error('Insufficient stock');
        }

        // Create order
        const order = await orderRepository.create(data);

        // Update stock
        await productService.updateProductStock(product.id, -data.quantity);

        return order;
    }
}
```

### 3. Keep Routes Thin

```typescript
// ✅ Good: Route just handles HTTP
router.post('/', async (req, res) => {
    const result = await service.doSomething(req.body);
    res.json(ApiResponse.success(result));
});

// ❌ Bad: Logic in route
router.post('/', async (req, res) => {
    if (!req.body.name) throw new Error('...');
    const existing = await repo.find(...);
    if (existing) throw new Error('...');
    // 50 more lines...
});
```

### 4. Return Business Objects, Not Responses

```typescript
// ✅ Good: Service returns data
class UserService {
    async createUser(data): Promise<User> {
        return user; // Just the data
    }
}

// ❌ Bad: Service returns HTTP response
class UserService {
    async createUser(data) {
        return {
            status: 201,
            body: {success: true, data: user}
        };
    }
}
```

---

## 📊 Statistics

### Code Reduction

| File          | Before    | After     | Reduction |
|---------------|-----------|-----------|-----------|
| `users.ts`    | 104 lines | 56 lines  | 46% ↓     |
| `auth.ts`     | 328 lines | 66 lines  | 80% ↓     |
| `products.ts` | 245 lines | 127 lines | 48% ↓     |

### Total:

- **Before:** 677 lines in routes
- **After:** 249 lines in routes + 450 lines in services
- **Routes reduced by:** 63% average
- **Better separation:** ✅
- **More testable:** ✅
- **More maintainable:** ✅

---

## 🎯 Summary

### Layer Responsibilities

| Layer            | Responsibility | Example                                      |
|------------------|----------------|----------------------------------------------|
| **Routes**       | HTTP handling  | Parse request, call service, format response |
| **Services**     | Business logic | Validation, orchestration, business rules    |
| **Repositories** | Data access    | CRUD operations, queries                     |

### The Pattern

```
Routes → Services → Repositories → Database
   ↓        ↓           ↓
 HTTP   Business     Data
       Logic      Access
```

---

## ✅ Implementation Complete!

Your application now has:

- ✅ Clean, thin routes (HTTP only)
- ✅ Business logic in services
- ✅ Data access in repositories
- ✅ Proper separation of concerns
- ✅ Easy to test
- ✅ Easy to maintain
- ✅ Professional architecture

**Routes berkurang 63%, logic lebih terorganisir!** 🎉
