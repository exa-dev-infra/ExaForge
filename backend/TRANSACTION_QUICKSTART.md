# ⚡ Quick Start - Database Transactions

## Basic Usage

```typescript
import Database from './config/database';

const db = Database.getInstance();

await db.withTransaction(async (client) => {
    await client.query('INSERT INTO users (name, email) VALUES ($1, $2)', ['John', 'john@example.com']);
    await client.query('INSERT INTO logs (action) VALUES ($1)', ['user_created']);
});
```

## In Routes

```typescript
import { Router } from 'express';
import Database from '../config/database';
import { ApiResponse } from '../types/response';
import ErrorHandler from '../middleware/errorHandler';

const router = Router();

router.post('/transfer', ErrorHandler.catchAsync(async (req, res) => {
    const { fromUserId, toUserId, amount } = req.body;
    const db = Database.getInstance();

    const result = await db.withTransaction(async (client) => {
        // Deduct from sender
        await client.query(
            'UPDATE accounts SET balance = balance - $1 WHERE user_id = $2',
            [amount, fromUserId]
        );

        // Add to receiver
        await client.query(
            'UPDATE accounts SET balance = balance + $1 WHERE user_id = $2',
            [amount, toUserId]
        );

        // Log the transaction
        const log = await client.query(
            'INSERT INTO transactions (from_user, to_user, amount) VALUES ($1, $2, $3) RETURNING *',
            [fromUserId, toUserId, amount]
        );

        return log.rows[0];
    });

    const response = ApiResponse.created(result, 'Transfer completed successfully');
    res.status(201).json(response);
}));
```

## Features

✅ **Automatic BEGIN** - Transaction starts automatically
✅ **Automatic COMMIT** - Commits if all succeed
✅ **Automatic ROLLBACK** - Rolls back on error
✅ **Automatic Client Release** - Client returned to pool
✅ **Error Propagation** - Works with error handler
✅ **Type Safe** - Full TypeScript support

## Common Patterns

### Transfer Money

```typescript
await db.withTransaction(async (client) => {
    await client.query('UPDATE accounts SET balance = balance - $1 WHERE id = $2', [amount, fromId]);
    await client.query('UPDATE accounts SET balance = balance + $1 WHERE id = $2', [amount, toId]);
});
```

### Create with Related Data

```typescript
await db.withTransaction(async (client) => {
    const user = await client.query('INSERT INTO users (name) VALUES ($1) RETURNING *', ['John']);
    await client.query('INSERT INTO addresses (user_id, city) VALUES ($1, $2)', [user.rows[0].id, 'NYC']);
    await client.query('INSERT INTO preferences (user_id, theme) VALUES ($1, $2)', [user.rows[0].id, 'dark']);
});
```

### Update with Lock

```typescript
await db.withTransaction(async (client) => {
    // Lock the row
    const product = await client.query('SELECT * FROM products WHERE id = $1 FOR UPDATE', [productId]);
    
    if (product.rows[0].stock < quantity) {
        throw new Error('Insufficient stock');
    }
    
    await client.query('UPDATE products SET stock = stock - $1 WHERE id = $2', [quantity, productId]);
});
```

## Error Handling

Errors are automatically caught and rolled back:

```typescript
await db.withTransaction(async (client) => {
    await client.query('INSERT INTO users (email) VALUES ($1)', ['duplicate@example.com']);
    // If email is duplicate, transaction is automatically rolled back
    // Error handler will format it as CONFLICT response
});
```

## Documentation

- **Complete Guide**: `TRANSACTION_GUIDE.md`
- **Example Routes**: `src/routes/transactions.ts`
- **Summary**: `TRANSACTION_SUMMARY.md`

## Examples

See `src/routes/transactions.ts` for complete examples:

- Money transfer
- Create user with related data
- Order processing with stock management
- Order cancellation with stock restoration
- Update with audit logging
