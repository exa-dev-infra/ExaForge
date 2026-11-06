# ⚡ Quick Reference - Error Handling

## 🎯 Base Response Format

```typescript
{
    success: boolean,
    message: string,
    data: T | null,
    code: string,
    timestamp: string
}
```

## 🚀 Quick Start

### 1. Import

```typescript
import { ApiResponse } from '../types/response';
import { NotFoundError, ValidationError } from '../types/errors';
import ErrorHandler from '../middleware/errorHandler';
```

### 2. Wrap Route

```typescript
router.get('/users', ErrorHandler.catchAsync(async (req, res) => {
    // Your code here - errors are automatically caught
}));
```

### 3. Return Success

```typescript
const response = ApiResponse.success(data);
res.json(response);
```

### 4. Throw Error

```typescript
throw new NotFoundError('User not found');
```

## 📝 Common Patterns

### Create Resource

```typescript
router.post('/users', ErrorHandler.catchAsync(async (req, res) => {
    const { name, email } = req.body;
    const result = await db.query(
        'INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *',
        [name, email]
    );
    const response = ApiResponse.created(result.rows[0]);
    res.status(201).json(response);
}));
```

### Get Resource

```typescript
router.get('/users/:id', ErrorHandler.catchAsync(async (req, res) => {
    const result = await db.query('SELECT * FROM users WHERE id = $1', [req.params.id]);
    if (result.rowCount === 0) throw new NotFoundError('User not found');
    const response = ApiResponse.success(result.rows[0]);
    res.json(response);
}));
```

### Update Resource

```typescript
router.put('/users/:id', ErrorHandler.catchAsync(async (req, res) => {
    const { name, email } = req.body;
    const result = await db.query(
        'UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING *',
        [name, email, req.params.id]
    );
    if (result.rowCount === 0) throw new NotFoundError('User not found');
    const response = ApiResponse.updated(result.rows[0]);
    res.json(response);
}));
```

### Delete Resource

```typescript
router.delete('/users/:id', ErrorHandler.catchAsync(async (req, res) => {
    const result = await db.query('DELETE FROM users WHERE id = $1', [req.params.id]);
    if (result.rowCount === 0) throw new NotFoundError('User not found');
    const response = ApiResponse.deleted();
    res.json(response);
}));
```

## 🎨 ApiResponse Methods

| Method                                 | HTTP Status | Use Case         |
|----------------------------------------|-------------|------------------|
| `success(data, message)`               | 200         | Success          |
| `created(data, message)`               | 201         | Resource created |
| `updated(data, message)`               | 200         | Resource updated |
| `deleted(message)`                     | 200         | Resource deleted |
| `notFound(message)`                    | 404         | Not found        |
| `badRequest(message, data)`            | 400         | Invalid input    |
| `unauthorized(message)`                | 401         | Auth required    |
| `forbidden(message)`                   | 403         | Access denied    |
| `paginated(items, page, limit, total)` | 200         | Paginated data   |

## 🚨 Custom Error Classes

| Class               | HTTP Status | Use Case                |
|---------------------|-------------|-------------------------|
| `NotFoundError`     | 404         | Resource not found      |
| `ValidationError`   | 400         | Validation failed       |
| `BadRequestError`   | 400         | Invalid request         |
| `UnauthorizedError` | 401         | Authentication required |
| `ForbiddenError`    | 403         | Access denied           |
| `ConflictError`     | 409         | Duplicate/Conflict      |
| `DatabaseError`     | 500         | Database error          |

## 🔧 PostgreSQL Errors (Auto-Handled)

| Error Code | Handled As            | Example                |
|------------|-----------------------|------------------------|
| 23505      | Unique violation      | Duplicate email        |
| 23503      | Foreign key violation | Invalid user_id        |
| 23502      | Not null violation    | Missing required field |
| 23514      | Check violation       | Invalid value          |

## 📝 Adding Constraint Mapping

Edit `src/constants/pgConstraints.ts`:

```typescript
export const UNIQUE_CONSTRAINT_FIELD_MAP: Record<string, string> = {
    users_email_key: 'email',
    products_sku_key: 'sku',  // Add this
}
```

## ✅ Response Examples

### Success

```json
{
    "success": true,
    "message": "User created successfully",
    "data": { "id": 1, "name": "John" },
    "code": "CREATED",
    "timestamp": "2026-01-17T12:00:00.000Z"
}
```

### Error

```json
{
    "success": false,
    "message": "The email 'john@example.com' already exists",
    "data": { "field": "email", "type": "unique_violation" },
    "code": "CONFLICT",
    "timestamp": "2026-01-17T12:00:00.000Z"
}
```

## 🧪 Testing

Use `api-test-error-handling.http` file:

```http
### Test duplicate email
POST http://localhost:3000/users
Content-Type: application/json

{"name":"John","email":"john@example.com"}

### Test not found
GET http://localhost:3000/users/999
```

## 📚 Full Documentation

- **Complete Guide**: `ERROR_HANDLING.md`
- **Summary**: `ERROR_HANDLING_SUMMARY.md`
- **Full Setup**: `FINAL_SUMMARY.md`

---

**Status**: ✅ Ready to use!
