---
name: typescript-worker
description: >-
  RabbitMQ consumer patterns, auto-reconnect, prefetch configuration, and message handling for the TypeScript worker template. Use when developing or modifying TypeScript background workers.
---

# TypeScript RabbitMQ Worker Skill Guide

This skill provides standard procedures, rules, and best practices for developing and maintaining the **TypeScript RabbitMQ Worker** template.

---

## 1. Architecture & Layout

```text
src/
├── config/
│   └── rabbitmq.config.ts       # Connection config (URL, queue, exchange, routingKey)
├── services/
│   └── rabbitmq.service.ts      # Connection manager, channel lifecycle, consume wrapper
├── workers/
│   └── message.worker.ts        # Message handler function & business logic
├── send-test-message.ts         # Test script to publish messages
└── index.ts                     # Worker bootstrap & graceful shutdown listeners
```

---

## 2. Core Conventions & Rules

### A. Consumer Lifecycle & Reconnect
- `RabbitMQService` handles connection establishment and channel creation.
- Set `prefetch` count (e.g. `10`) on the channel to avoid flooding a single instance.
- Graceful shutdown listens to `SIGINT` / `SIGTERM` and closes the AMQP channel cleanly.

### B. Message Acknowledgment
- **Always explicitly acknowledge or reject messages**:
  - Success: `channel.ack(msg)`
  - Recoverable Error: `channel.nack(msg, false, true)` (requeue for retry)
  - Poison Pill: `channel.nack(msg, false, false)` (drop or route to DLQ)
- Never use auto-acknowledgment (`{ noAck: true }`).

### C. Message Parsing & Handlers
- Handlers in `src/workers/` receive parsed payload objects:
  ```typescript
  export async function handleMessage(content: any): Promise<void> {
    // Process domain event
  }
  ```

---

## 3. Verification & Execution

```bash
# Install dependencies
npm install

# Run worker in development mode (ts-node-dev)
npm run dev

# Send test message
npm run send-test

# Build TypeScript
npm run build

# Start production worker
npm start
```
