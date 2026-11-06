# PostgreSQL, Redis, RabbitMQ Usage Guide

## 📘 Detailed Usage Examples

### 1. PostgreSQL Usage

#### Basic Operations

```typescript
import Database from './config/database';

const db = Database.getInstance();

// Simple SELECT query
const users = await db.query('SELECT * FROM users');
console.log(users.rows);

// Parameterized query (prevents SQL injection)
const user = await db.query(
    'SELECT * FROM users WHERE email = $1',
    ['user@example.com']
);

// INSERT with RETURNING
const newUser = await db.query(
    'INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *',
    ['John Doe', 'john@example.com']
);

// UPDATE
const updated = await db.query(
    'UPDATE users SET name = $1 WHERE id = $2 RETURNING *',
    ['Jane Doe', 1]
);

// DELETE
await db.query('DELETE FROM users WHERE id = $1', [1]);
```

#### Transactions

```typescript
import Database from './config/database';

const db = Database.getInstance();
const client = await db.getClient();

try {
    await client.query('BEGIN');

    // First operation
    const user = await client.query(
        'INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id',
        ['John', 'john@example.com']
    );

    // Second operation
    await client.query(
        'INSERT INTO products (name, user_id) VALUES ($1, $2)',
        ['Product 1', user.rows[0].id]
    );

    await client.query('COMMIT');
    console.log('Transaction completed successfully');
} catch (error) {
    await client.query('ROLLBACK');
    console.error('Transaction failed:', error);
    throw error;
} finally {
    client.release();
}
```

#### Complex Queries

```typescript
// JOIN queries
const result = await db.query(`
  SELECT u.name, u.email, p.name as product_name
  FROM users u
  LEFT JOIN products p ON u.id = p.user_id
  WHERE u.created_at > $1
  ORDER BY u.created_at DESC
  LIMIT $2
`, [new Date('2024-01-01'), 10]);

// Aggregation
const stats = await db.query(`
  SELECT 
    COUNT(*) as total_users,
    COUNT(DISTINCT email) as unique_emails,
    MAX(created_at) as latest_user
  FROM users
`);

// Subqueries
const activeUsers = await db.query(`
  SELECT * FROM users
  WHERE id IN (
    SELECT DISTINCT user_id 
    FROM products 
    WHERE created_at > NOW() - INTERVAL '30 days'
  )
`);
```

### 2. Redis Usage

#### String Operations

```typescript
import RedisClient from './config/redis';

const redis = RedisClient.getInstance();
await redis.connect();

// Set with expiration
await redis.set('session:user123', 'sessiondata', 3600); // 1 hour

// Get value
const session = await redis.get('session:user123');

// Check if exists
const exists = await redis.exists('session:user123');
if (exists) {
    console.log('Session exists');
}

// Delete
await redis.del('session:user123');

// Set expiration on existing key
await redis.expire('somekey', 300); // 5 minutes

// Get TTL (Time To Live)
const ttl = await redis.ttl('somekey');
console.log(`Key expires in ${ttl} seconds`);
```

#### Hash Operations (for Objects)

```typescript
// Store user data as hash
await redis.hSet('user:1000', 'name', 'John Doe');
await redis.hSet('user:1000', 'email', 'john@example.com');
await redis.hSet('user:1000', 'age', '30');

// Get single field
const name = await redis.hGet('user:1000', 'name');

// Get all fields
const userData = await redis.hGetAll('user:1000');
console.log(userData); // { name: 'John Doe', email: 'john@example.com', age: '30' }

// Delete field
await redis.hDel('user:1000', 'age');
```

#### Caching Pattern

```typescript
async function getCachedUser(userId: string) {
    const redis = RedisClient.getInstance();
    const db = Database.getInstance();

    // Try to get from cache
    const cacheKey = `user:${userId}`;
    const cached = await redis.get(cacheKey);

    if (cached) {
        console.log('Cache hit');
        return JSON.parse(cached);
    }

    // Cache miss - get from database
    console.log('Cache miss');
    const result = await db.query('SELECT * FROM users WHERE id = $1', [userId]);

    if (result.rows.length > 0) {
        const user = result.rows[0];
        // Cache for 1 hour
        await redis.set(cacheKey, JSON.stringify(user), 3600);
        return user;
    }

    return null;
}
```

#### Rate Limiting

```typescript
async function checkRateLimit(userId: string, limit: number = 10, window: number = 60) {
    const redis = RedisClient.getInstance();
    const key = `ratelimit:${userId}`;

    const current = await redis.get(key);

    if (!current) {
        // First request
        await redis.set(key, '1', window);
        return {allowed: true, remaining: limit - 1};
    }

    const count = parseInt(current);

    if (count >= limit) {
        const ttl = await redis.ttl(key);
        return {allowed: false, remaining: 0, resetIn: ttl};
    }

    await redis.set(key, (count + 1).toString(), await redis.ttl(key));
    return {allowed: true, remaining: limit - count - 1};
}
```

### 3. RabbitMQ Usage

#### Simple Queue Pattern

```typescript
import RabbitMQClient from './config/rabbitmq';

const rabbitmq = RabbitMQClient.getInstance();
await rabbitmq.connect();

// Producer: Send message
await rabbitmq.assertQueue('task_queue', {durable: true});
await rabbitmq.sendToQueue('task_queue', {
    task: 'send_email',
    data: {
        to: 'user@example.com',
        subject: 'Hello'
    }
});

// Consumer: Receive message
await rabbitmq.consume('task_queue', async (msg) => {
    if (msg) {
        const task = JSON.parse(msg.content.toString());
        console.log('Processing task:', task);

        try {
            // Process the task
            await processTask(task);

            // Acknowledge the message
            rabbitmq.ack(msg);
        } catch (error) {
            console.error('Task failed:', error);
            // Reject and requeue
            rabbitmq.nack(msg, false, true);
        }
    }
});
```

#### Publish/Subscribe Pattern (Fanout)

```typescript
// Publisher
await rabbitmq.assertExchange('logs', 'fanout', {durable: false});
await rabbitmq.publish('logs', '', {
    level: 'info',
    message: 'User logged in',
    timestamp: new Date()
});

// Subscriber 1: Console Logger
await rabbitmq.assertExchange('logs', 'fanout');
await rabbitmq.assertQueue('console_logs', {exclusive: true});
await rabbitmq.bindQueue('console_logs', 'logs', '');

await rabbitmq.consume('console_logs', (msg) => {
    if (msg) {
        const log = JSON.parse(msg.content.toString());
        console.log('[CONSOLE]', log);
        rabbitmq.ack(msg);
    }
});

// Subscriber 2: File Logger
await rabbitmq.assertQueue('file_logs', {exclusive: true});
await rabbitmq.bindQueue('file_logs', 'logs', '');

await rabbitmq.consume('file_logs', (msg) => {
    if (msg) {
        const log = JSON.parse(msg.content.toString());
        // Write to file
        fs.appendFileSync('app.log', JSON.stringify(log) + '\n');
        rabbitmq.ack(msg);
    }
});
```

#### Routing Pattern (Direct)

```typescript
// Publisher with routing
await rabbitmq.assertExchange('direct_logs', 'direct', {durable: false});

// Send error log
await rabbitmq.publish('direct_logs', 'error', {
    message: 'Database connection failed'
});

// Send info log
await rabbitmq.publish('direct_logs', 'info', {
    message: 'User logged in'
});

// Consumer: Only errors
await rabbitmq.assertQueue('error_logs');
await rabbitmq.bindQueue('error_logs', 'direct_logs', 'error');

await rabbitmq.consume('error_logs', (msg) => {
    if (msg) {
        console.log('[ERROR]', msg.content.toString());
        rabbitmq.ack(msg);
    }
});

// Consumer: All logs
await rabbitmq.assertQueue('all_logs');
await rabbitmq.bindQueue('all_logs', 'direct_logs', 'error');
await rabbitmq.bindQueue('all_logs', 'direct_logs', 'info');
await rabbitmq.bindQueue('all_logs', 'direct_logs', 'warning');
```

#### Topic Pattern (Advanced Routing)

```typescript
// Publisher
await rabbitmq.assertExchange('topic_logs', 'topic');

await rabbitmq.publish('topic_logs', 'user.created', {
    userId: 123,
    email: 'user@example.com'
});

await rabbitmq.publish('topic_logs', 'order.created', {
    orderId: 456,
    amount: 100
});

// Consumer: All user events
await rabbitmq.assertQueue('user_events');
await rabbitmq.bindQueue('user_events', 'topic_logs', 'user.*');

// Consumer: All created events
await rabbitmq.assertQueue('created_events');
await rabbitmq.bindQueue('created_events', 'topic_logs', '*.created');

// Consumer: All events
await rabbitmq.assertQueue('all_events');
await rabbitmq.bindQueue('all_events', 'topic_logs', '#');
```

#### Priority Queue

```typescript
// Create priority queue
await rabbitmq.assertQueue('priority_tasks', {
    durable: true,
    maxPriority: 10
});

// Send with priority
const channel = rabbitmq.getChannel();
channel.sendToQueue(
    'priority_tasks',
    Buffer.from(JSON.stringify({task: 'urgent'})),
    {priority: 10}
);

channel.sendToQueue(
    'priority_tasks',
    Buffer.from(JSON.stringify({task: 'normal'})),
    {priority: 5}
);
```

#### Dead Letter Queue

```typescript
// Main queue with DLQ
await rabbitmq.assertQueue('main_queue', {
    durable: true,
    deadLetterExchange: '',
    deadLetterRoutingKey: 'dead_letter_queue'
});

// Dead letter queue
await rabbitmq.assertQueue('dead_letter_queue', {durable: true});

// Consumer with retry logic
await rabbitmq.consume('main_queue', (msg) => {
    if (msg) {
        try {
            const data = JSON.parse(msg.content.toString());
            // Process...
            rabbitmq.ack(msg);
        } catch (error) {
            // Reject without requeue - goes to DLQ
            rabbitmq.nack(msg, false, false);
        }
    }
});
```

### 4. Complete Example: User Registration Flow

```typescript
import express from 'express';
import Database from './config/database';
import RedisClient from './config/redis';
import RabbitMQClient from './config/rabbitmq';

const router = express.Router();

router.post('/register', async (req, res) => {
    const {name, email, password} = req.body;
    const db = Database.getInstance();
    const redis = RedisClient.getInstance();
    const rabbitmq = RabbitMQClient.getInstance();

    try {
        // 1. Check cache for existing email
        const cacheKey = `email_exists:${email}`;
        const cached = await redis.get(cacheKey);

        if (cached === 'true') {
            return res.status(400).json({error: 'Email already exists'});
        }

        // 2. Check database
        const existing = await db.query(
            'SELECT id FROM users WHERE email = $1',
            [email]
        );

        if (existing.rows.length > 0) {
            await redis.set(cacheKey, 'true', 3600);
            return res.status(400).json({error: 'Email already exists'});
        }

        // 3. Create user
        const result = await db.query(
            'INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id, name, email',
            [name, email, password] // In production, hash the password!
        );

        const user = result.rows[0];

        // 4. Cache user data
        await redis.set(`user:${user.id}`, JSON.stringify(user), 3600);

        // 5. Send welcome email via queue
        await rabbitmq.assertQueue('email_queue');
        await rabbitmq.sendToQueue('email_queue', {
            to: user.email,
            subject: 'Welcome!',
            body: `Hello ${user.name}, welcome to our platform!`
        });

        // 6. Send notification
        await rabbitmq.assertQueue('notification_queue');
        await rabbitmq.sendToQueue('notification_queue', {
            userId: user.id,
            type: 'success',
            message: 'Registration successful'
        });

        res.status(201).json({
            success: true,
            user: {
                id: user.id,
                name: user.name,
                email: user.email
            }
        });

    } catch (error) {
        console.error('Registration error:', error);
        res.status(500).json({error: 'Registration failed'});
    }
});

export default router;
```

### 5. Performance Tips

#### PostgreSQL

- Use connection pooling (already configured)
- Create indexes on frequently queried columns
- Use prepared statements for repeated queries
- Analyze query performance with EXPLAIN

#### Redis

- Use appropriate data structures (strings, hashes, sets, sorted sets)
- Set expiration on cached data
- Use pipelining for multiple operations
- Monitor memory usage

#### RabbitMQ

- Use durable queues for important messages
- Set prefetch count to prevent overwhelming consumers
- Implement dead letter queues for failed messages
- Use appropriate exchange types for your use case

### 6. Monitoring and Health Checks

```typescript
// Health check endpoint
app.get('/health', async (req, res) => {
    const checks = {
        postgres: false,
        redis: false,
        rabbitmq: false
    };

    try {
        // Check PostgreSQL
        await Database.getInstance().query('SELECT 1');
        checks.postgres = true;
    } catch (error) {
        console.error('PostgreSQL health check failed:', error);
    }

    try {
        // Check Redis
        await RedisClient.getInstance().set('health', 'ok', 10);
        checks.redis = await RedisClient.getInstance().get('health') === 'ok';
    } catch (error) {
        console.error('Redis health check failed:', error);
    }

    try {
        // Check RabbitMQ
        RabbitMQClient.getInstance().getChannel();
        checks.rabbitmq = true;
    } catch (error) {
        console.error('RabbitMQ health check failed:', error);
    }

    const allHealthy = Object.values(checks).every(v => v === true);

    res.status(allHealthy ? 200 : 503).json({
        status: allHealthy ? 'healthy' : 'degraded',
        services: checks,
        timestamp: new Date().toISOString()
    });
});
```
