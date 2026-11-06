# Transaction Wrapper Documentation

## Overview

The `withTransaction` method provides a convenient way to execute multiple database operations within a single
transaction with automatic rollback on error.

## Usage

### Basic Example

```typescript
import Database from './config/database';

const db = Database.getInstance();

await db.withTransaction(async (client) => {
    // All queries within this block are part of the same transaction
    await client.query('INSERT INTO users (name, email) VALUES ($1, $2)', ['John', 'john@example.com']);
    await client.query('INSERT INTO logs (action) VALUES ($1)', ['user_created']);

    // If any query fails, all changes are automatically rolled back
});
```

### Example with Error Handling in Routes

```typescript
import {Router} from 'express';
import Database from '../config/database';
import {ApiResponse} from '../types/response';
import ErrorHandler from '../middleware/errorHandler';

const router = Router();

router.post('/transfer', ErrorHandler.catchAsync(async (req, res) => {
    const {fromUserId, toUserId, amount} = req.body;
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
        const logResult = await client.query(
            'INSERT INTO transactions (from_user, to_user, amount) VALUES ($1, $2, $3) RETURNING *',
            [fromUserId, toUserId, amount]
        );

        return logResult.rows[0];
    });

    const response = ApiResponse.created(result, 'Transfer completed successfully');
    res.status(201).json(response);
}));
```

### Example: Create User with Related Data

```typescript
router.post('/users/complete', ErrorHandler.catchAsync(async (req, res) => {
    const {name, email, addressData, preferences} = req.body;
    const db = Database.getInstance();

    const userData = await db.withTransaction(async (client) => {
        // Create user
        const userResult = await client.query(
            'INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *',
            [name, email]
        );
        const user = userResult.rows[0];

        // Create address
        await client.query(
            'INSERT INTO addresses (user_id, street, city, country) VALUES ($1, $2, $3, $4)',
            [user.id, addressData.street, addressData.city, addressData.country]
        );

        // Create preferences
        await client.query(
            'INSERT INTO user_preferences (user_id, theme, notifications) VALUES ($1, $2, $3)',
            [user.id, preferences.theme, preferences.notifications]
        );

        // Return the created user
        return user;
    });

    const response = ApiResponse.created(userData, 'User created with all related data');
    res.status(201).json(response);
}));
```

### Example: Conditional Operations

```typescript
router.post('/orders', ErrorHandler.catchAsync(async (req, res) => {
    const {userId, items} = req.body;
    const db = Database.getInstance();

    const order = await db.withTransaction(async (client) => {
        // Create order
        const orderResult = await client.query(
            'INSERT INTO orders (user_id, status) VALUES ($1, $2) RETURNING *',
            [userId, 'pending']
        );
        const orderId = orderResult.rows[0].id;

        // Add order items and update stock
        for (const item of items) {
            // Check stock
            const stockResult = await client.query(
                'SELECT stock FROM products WHERE id = $1 FOR UPDATE',
                [item.productId]
            );

            if (stockResult.rows[0].stock < item.quantity) {
                throw new Error(`Insufficient stock for product ${item.productId}`);
            }

            // Insert order item
            await client.query(
                'INSERT INTO order_items (order_id, product_id, quantity, price) VALUES ($1, $2, $3, $4)',
                [orderId, item.productId, item.quantity, item.price]
            );

            // Update stock
            await client.query(
                'UPDATE products SET stock = stock - $1 WHERE id = $2',
                [item.quantity, item.productId]
            );
        }

        // Return complete order
        const completeOrder = await client.query(
            'SELECT o.*, json_agg(oi.*) as items FROM orders o LEFT JOIN order_items oi ON o.id = oi.order_id WHERE o.id = $1 GROUP BY o.id',
            [orderId]
        );

        return completeOrder.rows[0];
    });

    const response = ApiResponse.created(order, 'Order created successfully');
    res.status(201).json(response);
}));
```

### Example with Try-Catch for Custom Error Handling

```typescript
router.post('/complex-operation', ErrorHandler.catchAsync(async (req, res) => {
    const db = Database.getInstance();

    try {
        const result = await db.withTransaction(async (client) => {
            // Your transactional operations
            const step1 = await client.query('...');
            const step2 = await client.query('...');

            return {step1, step2};
        });

        const response = ApiResponse.success(result, 'Operation completed');
        res.json(response);
    } catch (error) {
        if (error instanceof Error && error.message.includes('duplicate key')) {
            const response = ApiResponse.error('Record already exists', 'DUPLICATE');
            res.status(409).json(response);
        } else {
            throw error; // Let error handler middleware handle it
        }
    }
}));
```

## How it Works

1. **BEGIN**: Starts a new transaction
2. **Execute**: Runs your function with the client
3. **COMMIT**: If successful, commits all changes
4. **ROLLBACK**: If error occurs, automatically rolls back all changes
5. **Release**: Always releases the client back to the pool

## Benefits

✅ **Automatic Rollback**: No need to manually handle rollback
✅ **Clean Syntax**: No try-catch-finally boilerplate
✅ **Type Safe**: Full TypeScript support
✅ **Error Propagation**: Errors are properly thrown and can be caught by error middleware
✅ **Connection Management**: Client is automatically released

## Comparison

### Before (Manual Transaction)

```typescript
const client = await db.getClient();
try {
    await client.query('BEGIN');

    await client.query('INSERT INTO users ...');
    await client.query('INSERT INTO logs ...');

    await client.query('COMMIT');
} catch (error) {
    await client.query('ROLLBACK');
    throw error;
} finally {
    client.release();
}
```

### After (With Transaction Wrapper)

```typescript
await db.withTransaction(async (client) => {
    await client.query('INSERT INTO users ...');
    await client.query('INSERT INTO logs ...');
});
```

## Notes

- The transaction is automatically rolled back if any error occurs
- The client is always released back to the pool, even if an error occurs
- All queries within the transaction use the same client
- Use `FOR UPDATE` to lock rows when needed
- Transaction wrapper logs BEGIN, COMMIT, and ROLLBACK for debugging

## Error Handling

Errors thrown inside the transaction function will:

1. Trigger automatic ROLLBACK
2. Be re-thrown to be caught by error handler middleware
3. Follow the standard error handling flow (PostgreSQL errors, custom errors, etc.)

```typescript
await db.withTransaction(async (client) => {
    await client.query('INSERT INTO users (email) VALUES ($1)', ['duplicate@example.com']);
    // If email is duplicate, unique constraint error will be thrown
    // Transaction will be rolled back automatically
    // Error handler will format it as CONFLICT response
});
```

## Best Practices

1. ✅ Use transactions for operations that must succeed or fail together
2. ✅ Keep transactions as short as possible
3. ✅ Use `FOR UPDATE` when you need to lock rows
4. ✅ Let the error handler middleware handle errors
5. ✅ Return data from the transaction if needed
6. ❌ Don't make external API calls inside transactions
7. ❌ Don't perform long-running operations inside transactions

## Complete Example: Complex Order Processing

```typescript
router.post('/orders/process', ErrorHandler.catchAsync(async (req, res) => {
    const {userId, items, paymentInfo} = req.body;
    const db = Database.getInstance();

    const result = await db.withTransaction(async (client) => {
        // 1. Create order
        const order = await client.query(
            'INSERT INTO orders (user_id, total, status) VALUES ($1, $2, $3) RETURNING *',
            [userId, 0, 'processing']
        );
        const orderId = order.rows[0].id;
        let total = 0;

        // 2. Process each item
        for (const item of items) {
            // Check and lock product stock
            const product = await client.query(
                'SELECT id, price, stock FROM products WHERE id = $1 FOR UPDATE',
                [item.productId]
            );

            if (!product.rows[0]) {
                throw new NotFoundError(`Product ${item.productId} not found`);
            }

            if (product.rows[0].stock < item.quantity) {
                throw new BadRequestError(`Insufficient stock for product ${item.productId}`);
            }

            const itemTotal = product.rows[0].price * item.quantity;
            total += itemTotal;

            // Add order item
            await client.query(
                'INSERT INTO order_items (order_id, product_id, quantity, price) VALUES ($1, $2, $3, $4)',
                [orderId, item.productId, item.quantity, product.rows[0].price]
            );

            // Update stock
            await client.query(
                'UPDATE products SET stock = stock - $1 WHERE id = $2',
                [item.quantity, item.productId]
            );
        }

        // 3. Update order total
        await client.query(
            'UPDATE orders SET total = $1 WHERE id = $2',
            [total, orderId]
        );

        // 4. Create payment record
        await client.query(
            'INSERT INTO payments (order_id, amount, method, status) VALUES ($1, $2, $3, $4)',
            [orderId, total, paymentInfo.method, 'completed']
        );

        // 5. Update order status
        await client.query(
            'UPDATE orders SET status = $1 WHERE id = $2',
            ['completed', orderId]
        );

        // 6. Get complete order data
        const completeOrder = await client.query(`
            SELECT 
                o.*,
                json_agg(json_build_object(
                    'product_id', oi.product_id,
                    'quantity', oi.quantity,
                    'price', oi.price
                )) as items
            FROM orders o
            LEFT JOIN order_items oi ON o.id = oi.order_id
            WHERE o.id = $1
            GROUP BY o.id
        `, [orderId]);

        return completeOrder.rows[0];
    });

    const response = ApiResponse.created(result, 'Order processed successfully');
    res.status(201).json(response);
}));
```

This transaction ensures:

- All operations succeed or all fail (atomicity)
- Stock is properly locked to prevent race conditions
- Payment is only recorded if order completes
- Order status reflects actual state
