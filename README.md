# exaforge

> Fullstack project scaffolded via **Create Stack CLI**

## 🏗️ Architecture Overview

### 🔹 Backend: `Express.js (TypeScript)`
- **Location:** `./backend`
- **Stack:** Node.js / Express 5
- **Description:** Clean Architecture, PostgreSQL, Redis, RabbitMQ, JWT, Realtime (WS/SSE)
- **Default Port:** `http://localhost:5000`

### 🔹 Worker: `TypeScript Worker`
- **Location:** `./worker`
- **Stack:** Node.js / RabbitMQ
- **Description:** RabbitMQ Consumer (Ack/Nack, QoS Prefetch, Graceful Shutdown)

## 🚀 Quick Start Guide

### 🛠️ Setup Backend (`backend`)

```bash
cd backend
npm install
npm run dev
```

### 🛠️ Setup Worker (`worker`)

```bash
cd worker
npm install
npm run dev
```

