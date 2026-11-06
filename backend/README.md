# TypeScript Template with PostgreSQL, Redis, and RabbitMQ

A production-ready TypeScript application template with PostgreSQL database, Redis caching, and RabbitMQ message queue
integration.

## 📋 Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Database Migrations](#database-migrations)
- [Running the Application](#running-the-application)
- [API Endpoints](#api-endpoints)
- [Workers](#workers)
- [Usage Examples](#usage-examples)
- [Project Structure](#project-structure)

## ✨ Features

- ✅ TypeScript with strict mode
- ✅ PostgreSQL with connection pooling
- ✅ Redis for caching
- ✅ RabbitMQ for message queuing
- ✅ Database migrations with node-pg-migrate
- ✅ RESTful API with Express
- ✅ Background workers for async processing
- ✅ Graceful shutdown handling
- ✅ Development hot-reload with ts-node-dev

## 📦 Prerequisites

Before running this application, make sure you have the following installed:

- Node.js (v16 or higher)
- PostgreSQL (v12 or higher)
- Redis (v6 or higher)
- RabbitMQ (v3.8 or higher)

### Installing Services (Windows)

#### PostgreSQL

```bash
# Download from https://www.postgresql.org/download/windows/
# Or use Chocolatey
choco install postgresql
```

#### Redis

```bash
# Download from https://github.com/microsoftarchive/redis/releases
# Or use Chocolatey
choco install redis-64
```

#### RabbitMQ

```bash
# Download from https://www.rabbitmq.com/install-windows.html
# Or use Chocolatey
choco install rabbitmq
```

### Using Docker (Recommended)

```bash
# PostgreSQL
docker run -d --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15

# Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# RabbitMQ
docker run -d --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

## 🚀 Installation

1. Clone the repository and install dependencies:

```bash
npm install
```

2. Copy the environment file:

```bash
copy .env.example .env
```

3. Update `.env` with your configuration:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/mydb
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=mydb
DATABASE_USER=postgres
DATABASE_PASSWORD=password

REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0

RABBITMQ_URL=amqp://guest:guest@localhost:5672
RABBITMQ_HOST=localhost
RABBITMQ_PORT=5672
RABBITMQ_USER=guest
RABBITMQ_PASSWORD=guest

NODE_ENV=development
PORT=3000
```

## 🗄️ Database Migrations

### Create a new migration

```bash
npm run migrate:create -- your-migration-name
```

### Run migrations

```bash
npm run migrate:up
```

### Rollback migrations

```bash
npm run migrate:down
```

### Example: Initial setup migrations

```bash
npm run migrate:up
```

This will create the following tables:

- `users` - User data table
- `products` - Products table
- `email_logs` - Email logging table

## 🎯 Running the Application

### Development mode (with hot-reload)

```bash
npm run dev
```

### Build for production

```bash
npm run build
```

### Start production server

```bash
npm start
```

## 📡 API Endpoints

### Health Check

```http
GET /health
```

Response:

```json
{
  "status": "healthy",
  "services": {
    "database": "connected",
    "redis": "connected",
    "rabbitmq": "connected"
  },
  "timestamp": "2026-01-17T10:00:00.000Z"
}
```

### Database Example

```http
GET /db-example
```

### Redis Example

```http
POST /redis-example
Content-Type: application/json

{
  "key": "test-key",
  "value": "test-value",
  "expiry": 3600
}
```

### RabbitMQ Example

```http
POST /rabbitmq-send
Content-Type: application/json

{
  "queue": "email_queue",
  "message": {
    "to": "user@example.com",
    "subject": "Test Email",
    "body": "This is a test message"
  }
}
```

### Users API

#### Get all users

```http
GET /users
```

#### Get user by ID

```http
GET /users/:id
```

#### Create user

```http
POST /users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

#### Update user

```http
PUT /users/:id
Content-Type: application/json

{
  "name": "Jane Doe",
  "email": "jane@example.com"
}
```

#### Delete user

```http
DELETE /users/:id
```

## 👷 Workers

Workers process background jobs from RabbitMQ queues.

### Email Worker

Processes email sending jobs from the `email_queue`.

```bash
npx tsx src/workers/emailWorker.ts
```

### Notification Worker

Processes notifications from the `notification_queue`.

```bash
npx tsx src/workers/notificationWorker.ts
```

## 📚 Usage Examples

### PostgreSQL Usage

```typescript
import Database from './config/database';

const db = Database.getInstance();

// Simple query
const result = await db.query('SELECT * FROM users WHERE id = $1', [userId]);

// Using connection pool
const client = await db.getClient();
try {
    await client.query('BEGIN');
    await client.query('INSERT INTO users (name, email) VALUES ($1, $2)', ['John', 'john@example.com']);
    await client.query('COMMIT');
} catch (error) {
    await client.query('ROLLBACK');
    throw error;
} finally {
    client.release();
}
```

### Redis Usage

```typescript
import RedisClient from './config/redis';

const redis = RedisClient.getInstance();
await redis.connect();

// String operations
await redis.set('key', 'value', 3600); // with expiry in seconds
const value = await redis.get('key');

// Hash operations
await redis.hSet('user:1', 'name', 'John');
await redis.hSet('user:1', 'email', 'john@example.com');
const userData = await redis.hGetAll('user:1');

// Check if key exists
const exists = await redis.exists('key');

// Delete key
await redis.del('key');

// Set expiration
await redis.expire('key', 3600);
```

### RabbitMQ Usage

```typescript
import RabbitMQClient from './config/rabbitmq';

const rabbitmq = RabbitMQClient.getInstance();
await rabbitmq.connect();

// Simple Queue
await rabbitmq.assertQueue('my_queue');
await rabbitmq.sendToQueue('my_queue', {data: 'message'});

// Consume messages
await rabbitmq.consume('my_queue', async (msg) => {
    if (msg) {
        const content = JSON.parse(msg.content.toString());
        console.log('Received:', content);

        // Process message...

        // Acknowledge
        rabbitmq.ack(msg);
    }
});

// Exchange and Routing
await rabbitmq.assertExchange('logs', 'fanout');
await rabbitmq.assertQueue('log_queue');
await rabbitmq.bindQueue('log_queue', 'logs', '');
await rabbitmq.publish('logs', '', {log: 'message'});

// Topic Exchange
await rabbitmq.assertExchange('topics', 'topic');
await rabbitmq.bindQueue('queue1', 'topics', 'user.*');
await rabbitmq.publish('topics', 'user.created', {userId: 123});
```

## 📁 Project Structure

```
typescript/
├── src/
│   ├── config/
│   │   ├── database.ts      # PostgreSQL configuration
│   │   ├── redis.ts         # Redis configuration
│   │   └── rabbitmq.ts      # RabbitMQ configuration
│   ├── routes/
│   │   ├── index.ts         # Main routes
│   │   └── users.ts         # User routes
│   ├── workers/
│   │   ├── emailWorker.ts   # Email processing worker
│   │   └── notificationWorker.ts  # Notification worker
│   └── app.ts               # Main application file
├── migrations/
│   ├── 1737155000000_create-users-table.ts
│   ├── 1737155100000_create-products-table.ts
│   └── 1737155200000_create-email-logs-table.ts
├── public/                  # Static files
├── .env                     # Environment variables
├── .env.example            # Example environment variables
├── package.json
└── tsconfig.json
```

## 🔧 Configuration Files

### Database Migration Config

Create `.node-pg-migraterc` (optional):

```json
{
  "database-url-var": "DATABASE_URL",
  "migrations-dir": "migrations",
  "dir": "migrations",
  "direction": "up",
  "count": 1,
  "timestamp": true,
  "migration-file-language": "ts"
}
```

## 🛠️ Development Tips

### Running Multiple Workers

You can run multiple workers simultaneously:

```bash
# Terminal 1: Run the main app
npm run dev

# Terminal 2: Run email worker
npx tsx src/workers/emailWorker.ts

# Terminal 3: Run notification worker
npx tsx src/workers/notificationWorker.ts
```

### Testing Services

Test individual services:

```bash
# Test PostgreSQL
npm run dev
curl http://localhost:3000/db-example

# Test Redis
curl -X POST http://localhost:3000/redis-example -H "Content-Type: application/json" -d "{\"key\":\"test\",\"value\":\"hello\",\"expiry\":60}"

# Test RabbitMQ
curl -X POST http://localhost:3000/rabbitmq-send -H "Content-Type: application/json" -d "{\"queue\":\"email_queue\",\"message\":{\"to\":\"test@example.com\",\"subject\":\"Hello\",\"body\":\"Test\"}}"
```

## 🐛 Troubleshooting

### PostgreSQL connection issues

- Make sure PostgreSQL is running
- Check DATABASE_URL in .env
- Verify database exists: `createdb mydb`

### Redis connection issues

- Make sure Redis is running: `redis-cli ping`
- Check REDIS_HOST and REDIS_PORT in .env

### RabbitMQ connection issues

- Make sure RabbitMQ is running
- Access management UI: http://localhost:15672
- Default credentials: guest/guest

## 📄 License

MIT

## 👥 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
