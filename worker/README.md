# RabbitMQ Worker TypeScript

Worker TypeScript untuk consume messages dari RabbitMQ.

## Fitur

- ✅ Consumer RabbitMQ dengan TypeScript
- ✅ Auto-reconnect jika koneksi terputus
- ✅ Message acknowledgment
- ✅ Error handling dengan requeue
- ✅ Konfigurasi via environment variables
- ✅ Prefetch count untuk fair dispatch
- ✅ Graceful shutdown
- ✅ Type-safe dengan TypeScript

## Prerequisites

- Node.js v18 atau lebih tinggi
- RabbitMQ server (lokal atau remote)

## Instalasi

1. Install dependencies:

```bash
npm install
```

2. Copy file `.env.example` ke `.env`:

```bash
copy .env.example .env
```

3. Edit file `.env` sesuai konfigurasi RabbitMQ Anda:

```env
RABBITMQ_URL=amqp://localhost:5672
RABBITMQ_QUEUE=my_queue
RABBITMQ_EXCHANGE=my_exchange
RABBITMQ_EXCHANGE_TYPE=direct
RABBITMQ_ROUTING_KEY=my_routing_key
PREFETCH_COUNT=10
RETRY_DELAY=5000
```

## Cara Menggunakan

### Development Mode

```bash
npm run dev
```

### Production Mode

1. Build project:

```bash
npm run build
```

2. Jalankan worker:

```bash
npm start
```

### Watch Mode (Auto-rebuild)

```bash
npm run watch
```

## Struktur Project

```
worker-typescript/
├── src/
│   ├── config/
│   │   └── rabbitmq.config.ts    # Konfigurasi RabbitMQ
│   ├── services/
│   │   └── rabbitmq.service.ts   # Service untuk koneksi RabbitMQ
│   ├── workers/
│   │   └── message.worker.ts     # Handler untuk process message
│   └── index.ts                   # Entry point aplikasi
├── dist/                          # Compiled JavaScript files
├── .env                           # Environment variables
├── .env.example                   # Contoh environment variables
├── package.json
├── tsconfig.json
└── README.md
```

## Kustomisasi Message Handler

Edit file `src/workers/message.worker.ts` untuk mengubah cara processing message:

```typescript
export async function handleMessage(content: any, message: ConsumeMessage): Promise<void> {
    // Business logic Anda di sini
    console.log('Processing:', content);

    // Contoh: Simpan ke database
    // await database.save(content);

    // Contoh: Panggil external API
    // await fetch('https://api.example.com', { ... });
}
```

## Error Handling

Worker ini menggunakan strategi:

- **Acknowledge (ACK)**: Message berhasil diproses
- **Negative Acknowledge (NACK) + Requeue**: Message gagal diproses, akan dicoba lagi

Jika ada error saat processing, message akan otomatis di-requeue untuk diproses ulang.

## Graceful Shutdown

Worker akan melakukan graceful shutdown saat menerima signal SIGINT (Ctrl+C) atau SIGTERM:

- Tutup channel RabbitMQ
- Tutup connection
- Exit process dengan clean

## Testing dengan RabbitMQ

### Mengirim Test Message

Anda bisa menggunakan RabbitMQ Management UI atau script berikut:

```javascript
// send-message.js
const amqp = require('amqplib');

async function sendMessage() {
    const connection = await amqp.connect('amqp://localhost:5672');
    const channel = await connection.createChannel();

    const exchange = 'my_exchange';
    const routingKey = 'my_routing_key';

    await channel.assertExchange(exchange, 'direct', {durable: true});

    const message = {
        id: 1,
        name: 'Test Message',
        timestamp: new Date().toISOString()
    };

    channel.publish(
        exchange,
        routingKey,
        Buffer.from(JSON.stringify(message)),
        {persistent: true}
    );

    console.log('Message sent:', message);

    await channel.close();
    await connection.close();
}

sendMessage();
```

## Troubleshooting

### Connection Error

Jika tidak bisa connect ke RabbitMQ:

1. Pastikan RabbitMQ server sudah running
2. Cek URL RabbitMQ di file `.env`
3. Pastikan port 5672 tidak terblokir firewall

### Message Tidak Terproses

1. Cek queue name dan routing key sudah benar
2. Pastikan exchange type sesuai
3. Lihat RabbitMQ Management UI untuk status queue

## License

ISC
