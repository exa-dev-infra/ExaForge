# AGENTS.md — TypeScript RabbitMQ Worker Guidelines

Project rules for AI coding agents working in the TypeScript RabbitMQ Worker template.

---

## 1. Project Overview

- Background consumer for RabbitMQ built in TypeScript using `amqplib`.
- Features connection lifecycle management, auto-reconnect logic, manual message acknowledgment, and graceful shutdown listeners.
- Entry point: `src/index.ts`.

---

## 2. Important Commands

- `npm run dev` — Run worker in development mode with `ts-node-dev`.
- `npm run send-test` — Publish test message to queue.
- `npm run build` — Compile TypeScript to `dist/`.
- `npm start` — Run compiled JavaScript worker.

---

## 3. Core Rules & Conventions

- **Manual Acknowledgment**: Explicitly call `channel.ack(msg)` on success or `channel.nack(msg, false, true)` on failure. Never set `noAck: true`.
- **Fair Dispatch**: Configure channel `prefetch` to distribute tasks fairly among workers.
- **Graceful Shutdown**: Hook into `SIGINT` and `SIGTERM` to close channel and connection cleanly.
- **Strict Typing**: Type all message payloads and handler interfaces in TypeScript.
