# Panduan Penggunaan RabbitMQ Worker TypeScript

## Setup Awal

### 1. Install Dependencies

```bash
npm install
```

### 2. Konfigurasi Environment

Copy file `.env.example` menjadi `.env`:

```bash
copy .env.example .env
```

Edit file `.env` sesuai kebutuhan:

```env
RABBITMQ_URL=amqp://localhost:5672
RABBITMQ_QUEUE=my_queue
RABBITMQ_EXCHANGE=my_exchange
RABBITMQ_EXCHANGE_TYPE=direct
RABBITMQ_ROUTING_KEY=my_routing_key
PREFETCH_COUNT=10
RETRY_DELAY=5000
```

### 3. Setup RabbitMQ Server

#### Opsi A: Menggunakan Docker (Recommended)

```bash
docker-compose up -d
```

RabbitMQ Management UI akan tersedia di: http://localhost:15672

- Username: `guest`
- Password: `guest`

#### Opsi B: Install RabbitMQ Manual

Download dan install dari: https://www.rabbitmq.com/download.html

## Menjalankan Worker

### Development Mode (dengan hot reload)

```bash
npm run dev
```

### Production Mode

1. Build dulu:

```bash
npm run build
```

2. Jalankan:

```bash
npm start
```

## Testing Worker

### 1. Pastikan RabbitMQ sudah running

```bash
# Jika pakai Docker
docker-compose ps

# Atau cek di http://localhost:15672
```

### 2. Jalankan Worker

Di terminal pertama:

```bash
npm run dev
```

### 3. Kirim Test Message

Di terminal kedua:

```bash
npm run test:send
```

Anda akan melihat output di terminal worker yang memproses message.

## Struktur Project Dijelaskan

### src/config/rabbitmq.config.ts

File konfigurasi yang membaca environment variables dan menyediakan config object untuk RabbitMQ connection.

### src/services/rabbitmq.service.ts

Service utama yang menangani:

- Koneksi ke RabbitMQ
- Setup exchange dan queue
- Consume messages
- Auto-reconnect jika koneksi terputus
- Error handling

### src/workers/message.worker.ts

Handler untuk memproses message. **Edit file ini** untuk mengimplementasikan business logic Anda:

```typescript
export async function handleMessage(content: any, message: ConsumeMessage): Promise<void> {
    console.log('Processing message...');

    // Business logic Anda di sini
    // Contoh:
    // - Simpan ke database
    // - Panggil external API
    // - Transform data
    // - Kirim notifikasi

    console.log('Message processed successfully');
}
```

### src/index.ts

Entry point aplikasi yang:

- Inisialisasi RabbitMQ service
- Mulai consume messages
- Handle graceful shutdown

### src/send-test-message.ts

Utility script untuk mengirim test messages ke RabbitMQ.

## Kustomisasi Worker

### Mengubah Message Handler

Edit `src/workers/message.worker.ts`:

```typescript
import {ConsumeMessage} from 'amqplib';
import {database} from './database'; // contoh

export async function handleMessage(content: any, message: ConsumeMessage): Promise<void> {
    try {
        // Validasi message
        if (!content.userId || !content.action) {
            throw new Error('Invalid message format');
        }

        // Simpan ke database
        await database.users.update({
            id: content.userId,
            lastAction: content.action,
            updatedAt: new Date()
        });

        // Panggil external API
        await fetch('https://api.example.com/webhook', {
            method: 'POST',
            body: JSON.stringify(content)
        });

        console.log(`Processed action ${content.action} for user ${content.userId}`);
    } catch (error) {
        console.error('Error processing message:', error);
        throw error; // Will trigger requeue
    }
}
```

### Mengubah Exchange Type

RabbitMQ mendukung beberapa exchange types:

1. **direct** (default): Routing berdasarkan routing key exact match
2. **topic**: Routing berdasarkan pattern matching
3. **fanout**: Broadcast ke semua queue yang bound
4. **headers**: Routing berdasarkan message headers

Edit di `.env`:

```env
RABBITMQ_EXCHANGE_TYPE=topic
RABBITMQ_ROUTING_KEY=user.*.created
```

### Multiple Workers

Untuk scale horizontally, jalankan multiple instance worker:

```bash
# Terminal 1
npm run dev

# Terminal 2
npm run dev

# Terminal 3
npm run dev
```

Messages akan otomatis didistribusikan ke semua worker (round-robin).

## Error Handling

Worker ini menggunakan strategi:

### Message Berhasil Diproses

- `channel.ack(message)` - Message dihapus dari queue

### Message Gagal Diproses

- `channel.nack(message, false, true)` - Message di-requeue untuk dicoba lagi

### Connection Error

- Auto-reconnect dengan delay (default 5 detik)
- Retry indefinitely sampai koneksi berhasil

## Monitoring

### 1. RabbitMQ Management UI

Buka http://localhost:15672 untuk melihat:

- Queue status
- Message rate
- Consumer count
- Connection details

### 2. Console Logs

Worker akan print log untuk:

- Connection status
- Message received
- Message processed
- Errors

### 3. Custom Monitoring (Optional)

Tambahkan monitoring library seperti:

- Prometheus + Grafana
- Datadog
- New Relic

## Tips Best Practices

### 1. Idempotent Operations

Pastikan message handler bersifat idempotent (bisa dijalankan multiple kali dengan hasil yang sama):

```typescript
export async function handleMessage(content: any, message: ConsumeMessage): Promise<void> {
    // Check if already processed
    const exists = await database.processedMessages.findOne({
        messageId: message.properties.messageId
    });

    if (exists) {
        console.log('Message already processed, skipping...');
        return;
    }

    // Process message
    await processMessage(content);

    // Mark as processed
    await database.processedMessages.create({
        messageId: message.properties.messageId,
        processedAt: new Date()
    });
}
```

### 2. Set Message TTL

Tambahkan TTL untuk messages yang tidak perlu diproses selamanya:

```typescript
channel.publish(
    exchange,
    routingKey,
    Buffer.from(JSON.stringify(message)),
    {
        persistent: true,
        expiration: '3600000' // 1 hour in milliseconds
    }
);
```

### 3. Dead Letter Queue

Setup DLQ untuk message yang gagal berkali-kali:

```typescript
await channel.assertQueue(this.config.queue, {
    durable: true,
    arguments: {
        'x-dead-letter-exchange': 'dlx-exchange',
        'x-dead-letter-routing-key': 'dlx-routing-key'
    }
});
```

### 4. Prefetch Count

Sesuaikan `PREFETCH_COUNT` berdasarkan:

- Resource server (CPU, Memory)
- Processing time per message
- Jumlah worker instances

Aturan umum:

- Processing cepat (<100ms): 50-100
- Processing medium (100ms-1s): 10-20
- Processing lambat (>1s): 1-5

### 5. Graceful Shutdown

Worker sudah implement graceful shutdown. Saat CTRL+C:

1. Stop consume messages baru
2. Finish processing current messages
3. Close channel dan connection
4. Exit

## Troubleshooting

### Worker tidak bisa connect ke RabbitMQ

**Solusi:**

1. Cek RabbitMQ server running: `docker-compose ps`
2. Cek URL di `.env` benar
3. Cek firewall tidak block port 5672
4. Cek credentials (username/password) benar

### Messages tidak terproses

**Solusi:**

1. Cek queue name di `.env` sama dengan yang digunakan producer
2. Cek routing key match dengan binding
3. Cek di RabbitMQ Management UI apakah message masuk ke queue
4. Cek console log untuk error messages

### Memory Usage Tinggi

**Solusi:**

1. Turunkan `PREFETCH_COUNT`
2. Optimize message handler (gunakan streaming untuk large data)
3. Add memory limit di Docker jika pakai container
4. Check for memory leaks

### Message Requeue Terus-menerus

**Solusi:**

1. Fix bug di message handler
2. Add retry limit counter
3. Setup Dead Letter Queue untuk failed messages
4. Log error detail untuk debugging

## Deployment

### Production Checklist

- [ ] Set environment variables yang benar
- [ ] Build project: `npm run build`
- [ ] Test di staging environment
- [ ] Setup monitoring dan alerting
- [ ] Configure log aggregation
- [ ] Setup auto-restart (PM2, systemd, Docker restart policy)
- [ ] Document runbook untuk on-call

### Menggunakan PM2

```bash
npm install -g pm2

# Build dulu
npm run build

# Start dengan PM2
pm2 start dist/index.js --name rabbitmq-worker

# Auto-restart on reboot
pm2 startup
pm2 save

# Monitor
pm2 monit
```

### Menggunakan Docker

Buat `Dockerfile`:

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

CMD ["npm", "start"]
```

Build dan run:

```bash
docker build -t rabbitmq-worker .
docker run -d --name worker --env-file .env rabbitmq-worker
```

## Bantuan Tambahan

Jika ada pertanyaan atau masalah, cek:

1. RabbitMQ Documentation: https://www.rabbitmq.com/documentation.html
2. amqplib Documentation: https://amqp-node.github.io/amqplib/
3. TypeScript Documentation: https://www.typescriptlang.org/docs/
