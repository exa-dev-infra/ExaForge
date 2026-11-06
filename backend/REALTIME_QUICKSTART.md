# SSE & WebSocket Quick Start Guide

## 🚀 Quick Setup (5 Minutes)

### 1. Install Dependencies (Already Done ✅)

```bash
npm install socket.io @socket.io/redis-adapter
npm install --save-dev @types/socket.io
```

### 2. Start Services

```bash
# Start Docker services (PostgreSQL, Redis, RabbitMQ)
npm run docker:up

# Start the server
npm run dev
```

You'll see:

```
✓ PostgreSQL connected
✓ Redis connected
✓ RabbitMQ connected
✓ SSE Service initialized with Redis PubSub
✓ WebSocket Service initialized with Redis adapter
🚀 Server is running on port 3000
📡 SSE endpoint: http://localhost:3000/sse/stream
🔌 WebSocket endpoint: ws://localhost:3000
```

---

## 📡 SSE Quick Test

### Option 1: Use Browser Demo

1. Open: http://localhost:3000/sse-demo.html
2. Login first to get JWT token: `POST /auth/login`
3. Enter your JWT token
4. Click "Connect"
5. Send test messages!

### Option 2: Use cURL

```bash
# Get JWT token first
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Connect to SSE stream (replace TOKEN)
curl -N -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:3000/sse/stream

# In another terminal, send test message
curl -X POST http://localhost:3000/sse/test \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message":"Hello SSE!"}'
```

---

## 🔌 WebSocket Quick Test

### Option 1: Use Browser Demo

1. Open: http://localhost:3000/websocket-demo.html
2. Enter your JWT token
3. Click "Connect"
4. Try:
    - Join room
    - Subscribe to channel
    - Send messages
    - Ping/pong test

### Option 2: Use JavaScript

```html
<!DOCTYPE html>
<html>
<head>
    <script src="https://cdn.socket.io/4.5.4/socket.io.min.js"></script>
</head>
<body>
<script>
    const socket = io('http://localhost:3000', {
        auth: {token: 'YOUR_JWT_TOKEN'}
    });

    socket.on('connect', () => {
        console.log('Connected:', socket.id);
    });

    socket.on('connected', (data) => {
        console.log('Welcome:', data);
    });

    socket.on('notification', (data) => {
        console.log('Notification:', data);
    });

    // Join room
    socket.emit('join:room', 'test-room');

    // Send message
    socket.emit('message', {text: 'Hello!'});
</script>
</body>
</html>
```

---

## 💻 Send Messages from Code

### Example: Send Notification After User Register

```typescript
// In authService.ts
import {RealtimeService} from '../services';

class AuthService {
    async register(data: RegisterDTO): Promise<AuthResponse> {
        // ... create user logic ...

        // Send welcome notification via SSE & WebSocket
        const realtimeService = RealtimeService.getInstance();
        await realtimeService.notifyUser(user.id, 'notification', {
            type: 'welcome',
            title: 'Welcome!',
            message: 'Your account has been created successfully',
            timestamp: new Date().toISOString()
        });

        return {user, tokens};
    }
}
```

### Example: Broadcast System Announcement

```typescript
// In any service
import {RealtimeService} from '../services';

const realtimeService = RealtimeService.getInstance();

// Broadcast to all users
await realtimeService.broadcastToAll('announcement', {
    type: 'maintenance',
    message: 'System will be down for 5 minutes',
    scheduledAt: '2026-01-18T10:00:00Z'
});
```

### Example: Send to Specific Channel

```typescript
// Chat room example
import {WebSocketService} from '../services';

const wsService = WebSocketService.getInstance();

// Send message to chat room
wsService.sendToRoom('room-123', 'chat:message', {
    userId: 'user-456',
    username: 'John Doe',
    message: 'Hello everyone!',
    timestamp: new Date().toISOString()
});
```

---

## 🔐 API Endpoints

All endpoints require authentication via JWT token in `Authorization: Bearer <token>` header.

### SSE Endpoints

| Method | Endpoint    | Description              |
|--------|-------------|--------------------------|
| GET    | /sse/stream | Connect to SSE stream    |
| GET    | /sse/stats  | Get SSE statistics       |
| POST   | /sse/test   | Send test message to you |

### Realtime API Endpoints

| Method | Endpoint                     | Description                  |
|--------|------------------------------|------------------------------|
| POST   | /realtime/notify/:userId     | Send to specific user        |
| POST   | /realtime/broadcast          | Broadcast to all             |
| POST   | /realtime/channel/:channel   | Send to channel              |
| GET    | /realtime/stats              | Get all statistics           |
| GET    | /realtime/online/:userId     | Check if user is online      |
| POST   | /realtime/disconnect/:userId | Disconnect user (admin only) |

---

## 📋 Common Use Cases

### 1. Real-time Notifications

```typescript
// When order is created
await realtimeService.notifyUser(userId, 'order:created', {
    orderId: order.id,
    status: 'processing',
    amount: order.total
});

// When payment is successful
await realtimeService.notifyUser(userId, 'payment:success', {
    transactionId: payment.id,
    amount: payment.amount
});
```

### 2. Live Dashboard Updates

```typescript
// Update dashboard stats every 5 seconds
setInterval(async () => {
    const stats = await getSystemStats();

    // Send to admin dashboard channel
    await realtimeService.sendToChannel('admin-dashboard', 'stats:update', {
        totalUsers: stats.users,
        activeOrders: stats.orders,
        revenue: stats.revenue,
        timestamp: new Date().toISOString()
    });
}, 5000);
```

### 3. Chat Application

```typescript
// User sends message
router.post('/chat/:roomId/message', async (req, res) => {
    const {roomId} = req.params;
    const {message} = req.body;
    const userId = (req as any).user?.userId;

    // Save to database
    const chatMessage = await chatRepository.create({
        roomId,
        userId,
        message
    });

    // Broadcast to room
    wsService.sendToRoom(roomId, 'chat:message', {
        id: chatMessage.id,
        userId,
        username: req.user.name,
        message,
        timestamp: chatMessage.createdAt
    });

    res.json(ApiResponse.success(chatMessage));
});
```

### 4. User Presence

```typescript
// Check who's online
router.get('/users/online', async (req, res) => {
    const stats = await wsService.getStats();

    res.json(ApiResponse.success({
        onlineCount: stats.totalUsers,
        onlineUsers: stats.onlineUsers
    }));
});

// Check specific user
router.get('/users/:id/online', async (req, res) => {
    const isOnline = wsService.isUserOnline(req.params.id);

    res.json(ApiResponse.success({
        userId: req.params.id,
        isOnline
    }));
});
```

### 5. Progress Updates

```typescript
// Long-running task with progress updates
async function processLargeFile(userId: string, fileId: string) {
    const chunks = 100;

    for (let i = 0; i <= chunks; i++) {
        // Process chunk
        await processChunk(i);

        // Send progress update
        await realtimeService.notifyUser(userId, 'file:progress', {
            fileId,
            progress: (i / chunks) * 100,
            status: i === chunks ? 'completed' : 'processing'
        });

        await sleep(100);
    }
}
```

---

## 🧪 Testing

### Test SSE Connection

```javascript
// Client-side code
const eventSource = new EventSource('http://localhost:3000/sse/stream', {
    headers: {
        'Authorization': 'Bearer YOUR_TOKEN'
    }
});

eventSource.addEventListener('notification', (e) => {
    console.log('Notification:', JSON.parse(e.data));
});

eventSource.addEventListener('test', (e) => {
    console.log('Test:', JSON.parse(e.data));
});

eventSource.onerror = (error) => {
    console.error('SSE Error:', error);
};
```

### Test WebSocket Connection

```javascript
// Client-side code
const socket = io('http://localhost:3000', {
    auth: {token: 'YOUR_TOKEN'}
});

socket.on('connect', () => {
    console.log('Connected!', socket.id);

    // Join room
    socket.emit('join:room', 'test-room');

    // Subscribe to channel
    socket.emit('subscribe', 'news');
});

socket.on('notification', (data) => {
    console.log('Notification:', data);
});

socket.on('joined:room', (data) => {
    console.log('Joined room:', data);
});

socket.emit('message', {text: 'Hello!'});
```

---

## 🔧 Configuration

### Environment Variables

Add to `.env`:

```env
# Realtime Configuration
CORS_ORIGIN=http://localhost:3000

# Redis (required for scaling)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0
```

### Nginx for Production

```nginx
upstream websocket_backend {
    ip_hash; # Sticky sessions for WebSocket
    server localhost:3001;
    server localhost:3002;
}

server {
    listen 80;
    
    location / {
        proxy_pass http://websocket_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
    
    location /sse/ {
        proxy_pass http://websocket_backend;
        proxy_set_header Connection '';
        proxy_http_version 1.1;
        chunked_transfer_encoding off;
        proxy_buffering off;
        proxy_cache off;
    }
}
```

---

## 🌐 Horizontal Scaling

Your setup already supports horizontal scaling!

### Run Multiple Instances

```bash
# Terminal 1
PORT=3001 npm run dev

# Terminal 2
PORT=3002 npm run dev

# Terminal 3
PORT=3003 npm run dev
```

All instances will:

- Share Redis for message broadcasting
- Sync WebSocket connections via Redis adapter
- Work seamlessly behind a load balancer

### How It Works

```
Client A → Server 1 → Redis → Server 2 → Client B
Client C → Server 2 → Redis → Server 1 → Client A
```

Messages sent to any server are broadcast to all servers via Redis!

---

## 📊 Monitoring

### Get Statistics

```bash
# Get SSE stats
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:3000/sse/stats

# Get WebSocket stats
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:3000/realtime/stats
```

Response:

```json
{
  "success": true,
  "data": {
    "sse": {
      "totalUsers": 25,
      "totalConnections": 25
    },
    "websocket": {
      "totalUsers": 150,
      "totalConnections": 200,
      "onlineUsers": [
        "user1",
        "user2",
        ...
      ]
    }
  }
}
```

---

## 🐛 Troubleshooting

### SSE Not Connecting

1. **Check JWT token** - Make sure it's valid
2. **Check Redis** - Must be running
3. **Check CORS** - Set `CORS_ORIGIN` in .env
4. **Check nginx** - Disable buffering with `X-Accel-Buffering: no`

### WebSocket Not Connecting

1. **Check authentication** - Token in `auth.token` or `Authorization` header
2. **Check Redis adapter** - Redis must be running
3. **Check firewall** - Port 3000 must be open
4. **Try polling first** - Set `transports: ['polling', 'websocket']`

### Messages Not Received

1. **Check Redis connection** - All servers must connect to same Redis
2. **Check room/channel** - User must join room/subscribe to channel
3. **Check event names** - Must match on client and server

---

## 💡 Tips & Best Practices

1. **Always authenticate** - Verify JWT token before allowing connections
2. **Use rooms/channels** - More efficient than sending to individual users
3. **Handle reconnections** - Client should auto-reconnect on disconnect
4. **Rate limiting** - Prevent abuse with rate limits
5. **Compression** - Enable for large messages
6. **Monitoring** - Track connections, messages, latency
7. **Error handling** - Always handle errors gracefully
8. **Testing** - Test with multiple clients and servers

---

## ✅ You're Ready!

Your scalable SSE & WebSocket infrastructure is ready for production!

**Features:**

- ✅ SSE with Redis PubSub
- ✅ WebSocket with Socket.IO
- ✅ JWT Authentication
- ✅ Horizontal Scaling
- ✅ User/Room/Channel targeting
- ✅ Broadcasting
- ✅ Online presence tracking
- ✅ Production-ready

**Demo Pages:**

- http://localhost:3000/sse-demo.html
- http://localhost:3000/websocket-demo.html

**Documentation:**

- Full guide: `REALTIME_GUIDE.md`
- This quickstart: `REALTIME_QUICKSTART.md`

🎉 **Happy coding!**
