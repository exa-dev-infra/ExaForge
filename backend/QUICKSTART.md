# Quick Start Guide

## 🚀 Quick Start (5 Minutes)

### 1. Start Services dengan Docker

```bash
npm run docker:up
```

Ini akan menjalankan:

- PostgreSQL di port 5432
- Redis di port 6379
- RabbitMQ di port 5672 (management UI di 15672)

### 2. Install Dependencies

```bash
npm install
```

### 3. Setup Environment

File `.env` sudah tersedia dengan konfigurasi default.

### 4. Run Database Migrations

```bash
npm run migrate:up
```

### 5. Start Application

```bash
npm run dev
```

Server akan berjalan di `http://localhost:3000`

### 6. Test API

Buka browser atau gunakan curl:

```bash
# Health check
curl http://localhost:3000/health

# Test database
curl http://localhost:3000/db-example

# Create user
curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d "{\"name\":\"John Doe\",\"email\":\"john@example.com\"}"

# Get all users
curl http://localhost:3000/users
```

### 7. Run Workers (Optional)

Di terminal baru:

```bash
# Run email worker
npm run worker:email

# Atau run notification worker
npm run worker:notification

# Atau run semua workers sekaligus
npm run workers
```

### 8. Send Message to Queue

```bash
curl -X POST http://localhost:3000/rabbitmq-send \
  -H "Content-Type: application/json" \
  -d "{\"queue\":\"email_queue\",\"message\":{\"to\":\"test@example.com\",\"subject\":\"Hello\",\"body\":\"Test email\"}}"
```

Worker akan memproses message tersebut.

## 📊 Management UIs

- **RabbitMQ Management**: http://localhost:15672 (guest/guest)
- **API**: http://localhost:3000

## 🛑 Stop Services

```bash
npm run docker:down
```

## 📝 Available Commands

| Command                         | Description                              |
|---------------------------------|------------------------------------------|
| `npm run dev`                   | Start development server with hot-reload |
| `npm run build`                 | Build TypeScript to JavaScript           |
| `npm start`                     | Run production server                    |
| `npm run migrate:up`            | Run database migrations                  |
| `npm run migrate:down`          | Rollback last migration                  |
| `npm run migrate:create <name>` | Create new migration                     |
| `npm run worker:email`          | Run email worker                         |
| `npm run worker:notification`   | Run notification worker                  |
| `npm run workers`               | Run all workers                          |
| `npm run docker:up`             | Start Docker services                    |
| `npm run docker:down`           | Stop Docker services                     |
| `npm run docker:logs`           | View Docker logs                         |

## 🔧 Troubleshooting

### Port already in use

```bash
# Windows
netstat -ano | findstr :3000
taskkill /PID <PID> /F

# Check Docker services
docker ps
```

### Connection refused

```bash
# Check services status
docker-compose ps

# View logs
npm run docker:logs
```

### Migration errors

```bash
# Drop database and recreate
docker-compose down -v
npm run docker:up
npm run migrate:up
```

## 📚 Next Steps

1. Baca [README.md](README.md) untuk dokumentasi lengkap
2. Lihat [USAGE.md](USAGE.md) untuk contoh kode detail
3. Explore folder `src/` untuk melihat struktur kode
4. Customize sesuai kebutuhan project Anda

## 🎯 Example Flow

1. User register → API creates user in PostgreSQL
2. API caches user data in Redis
3. API sends welcome email job to RabbitMQ
4. Email worker processes the job
5. Worker logs email status to PostgreSQL

Semua ini terjadi secara asynchronous dan scalable!
