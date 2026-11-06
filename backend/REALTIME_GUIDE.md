# SSE & WebSocket - Scalable Real-time Communication Guide

## 🎯 Overview

Implementasi **SSE (Server-Sent Events)** dan **WebSocket** yang scalable menggunakan **Redis PubSub** dan **Socket.IO
Redis Adapter**. Mendukung horizontal scaling dengan multiple server instances.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLIENT APPLICATIONS                       │
│              (Browser, Mobile App, Desktop)                  │
└────────────┬────────────────────────────────┬────────────────┘
             │                                │
             ▼                                ▼
    ┌────────────────┐              ┌────────────────┐
    │  SSE Client    │              │  WS Client     │
    │  EventSource   │              │  Socket.IO     │
    └────────┬───────┘              └───────┬────────┘
             │                              │
             └──────────────┬───────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         │         Load Balancer (Nginx)       │
         └──────────┬──────────────────┬───────┘
                    │                  │
        ┌───────────▼─────┐   ┌───────▼──────────┐
        │  Server 1       │   │  Server 2        │
        │  Express + WS   │   │  Express + WS    │
        └───────┬─────────┘   └────────┬─────────┘
                │                      │
                └──────────┬───────────┘
                           │
                  ┌────────▼─────────┐
                  │  Redis PubSub    │
                  │  (Message Broker)│
                  └──────────────────┘
```

## 📊 Features

### SSE (Server-Sent Events)

- ✅ Unidirectional (Server → Client)
- ✅ HTTP/1.1 compatible
- ✅ Auto reconnect
- ✅ Simple implementation
- ✅ Redis PubSub untuk scaling
- ✅ User-specific streams
- ✅ Channel/Room support
- ✅ Broadcasting

### WebSocket (Socket.IO)

- ✅ Bidirectional (Server ↔ Client)
- ✅ Auto reconnect dengan fallback
- ✅ JWT Authentication
- ✅ Redis Adapter untuk scaling
- ✅ Rooms & Channels
- ✅ Namespace support
- ✅ Broadcasting
- ✅ Online user tracking

## 📁 File Structure

```
src/
├── services/
│   ├── sseService.ts          # SSE dengan Redis PubSub
│   ├── websocketService.ts    # WebSocket dengan Socket.IO
│   └── realtimeService.ts     # Unified facade untuk SSE + WS
│
└── routes/
    ├── sse.ts                 # SSE endpoints
    └── realtime.ts            # Realtime API endpoints
```

---

## 🚀 Quick Start

### 1. Environment Variables

Add to `.env`:

```env
# Realtime Configuration
CORS_ORIGIN=http://localhost:3000
```

### 2. Start Server

```bash
npm run dev
```

Server akan menjalankan:

- 📡 SSE endpoint: `http://localhost:3000/sse/stream`
- 🔌 WebSocket: `ws://localhost:3000`

---

## 📡 SSE Usage

### Client Side (JavaScript)

```javascript
// Connect to SSE
const eventSource = new EventSource('http://localhost:3000/sse/stream', {
    headers: {
        'Authorization': 'Bearer YOUR_JWT_TOKEN'
    }
});

// Listen to connection event
eventSource.addEventListener('connected', (e) => {
    const data = JSON.parse(e.data);
    console.log('Connected:', data);
});

// Listen to custom events
eventSource.addEventListener('notification', (e) => {
    const data = JSON.parse(e.data);
    console.log('Notification:', data);
});

// Listen to all messages
eventSource.onmessage = (e) => {
    console.log('Message:', e.data);
};

// Handle errors
eventSource.onerror = (error) => {
    console.error('SSE Error:', error);
};

// Close connection
eventSource.close();
```

### Server Side (Send Messages)

```typescript
import { SSEService } from '../services';

const sseService = SSEService.getInstance();

// Send to specific user
await sseService.sendToUser('user123', 'notification', {
    title: 'New Message',
    body: 'You have a new message'
});

// Broadcast to all
await sseService.broadcast('announcement', {
    message: 'Server maintenance in 10 minutes'
});

// Send to channel
await sseService.sendToChannel('chat-room-1', 'message', {
    user: 'John',
    text: 'Hello!'
});
```

---

## 🔌 WebSocket Usage

### Client Side (JavaScript)

```javascript
import io from 'socket.io-client';

// Connect to WebSocket
const socket = io('http://localhost:3000', {
    auth: {
        token: 'YOUR_JWT_TOKEN'
    },
    transports: ['websocket', 'polling']
});

// Connection events
socket.on('connect', () => {
    console.log('Connected:', socket.id);
});

socket.on('connected', (data) => {
    console.log('Welcome message:', data);
});

socket.on('disconnect', () => {
    console.log('Disconnected');
});

// Listen to custom events
socket.on('notification', (data) => {
    console.log('Notification:', data);
});

// Join room
socket.emit('join:room', 'room-123');

socket.on('joined:room', (data) => {
    console.log('Joined room:', data);
});

// Subscribe to channel
socket.emit('subscribe', 'news-channel');

socket.on('subscribed', (data) => {
    console.log('Subscribed to:', data);
});

// Send message
socket.emit('message', { text: 'Hello server!' });

socket.on('message:received', (data) => {
    console.log('Message received:', data);
});

// Ping-pong test
socket.emit('ping');

socket.on('pong', (data) => {
    console.log('Pong:', data);
});

// Disconnect
socket.disconnect();
```

### Server Side (Send Messages)

```typescript
import { WebSocketService } from '../services';

const wsService = WebSocketService.getInstance();

// Send to specific user
wsService.sendToUser('user123', 'notification', {
    title: 'New Order',
    orderId: 12345
});

// Send to room
wsService.sendToRoom('chat-room-1', 'message', {
    user: 'John',
    text: 'Hello everyone!'
});

// Send to channel
wsService.sendToChannel('news-channel', 'news', {
    title: 'Breaking News',
    content: '...'
});

// Broadcast to all
wsService.broadcast('announcement', {
    message: 'Server maintenance soon'
});

// Check if user is online
const isOnline = wsService.isUserOnline('user123');

// Disconnect user
await wsService.disconnectUser('user123', 'Kicked by admin');

// Get stats
const stats = await wsService.getStats();
console.log(stats);
// {
//   totalUsers: 150,
//   totalConnections: 200,
//   onlineUsers: ['user1', 'user2', ...]
// }
```

---

## 🎯 Unified Realtime Service

Gunakan `RealtimeService` untuk mengirim via SSE dan WebSocket sekaligus:

```typescript
import { RealtimeService } from '../services';

const realtimeService = RealtimeService.getInstance();

// Send via both SSE and WebSocket
await realtimeService.notifyUser('user123', 'notification', {
    title: 'Payment Success',
    amount: 100000
});

// Broadcast to all
await realtimeService.broadcastToAll('maintenance', {
    message: 'System will be down for 5 minutes'
});

// Send to channel (both SSE and WS)
await realtimeService.sendToChannel('support', 'ticket', {
    ticketId: 789,
    status: 'new'
});

// Or use specific transport
await realtimeService.sendSSE('user123', 'event', data); // SSE only
realtimeService.sendWebSocket('user123', 'event', data); // WS only

// Get stats
const stats = await realtimeService.getStats();
console.log(stats);
// {
//   sse: { totalUsers: 50, totalConnections: 50 },
//   websocket: { totalUsers: 150, totalConnections: 200, onlineUsers: [...] }
// }
```

---

## 🔐 API Endpoints

All endpoints require JWT authentication.

### SSE Endpoints

#### 1. Connect to SSE Stream

```http
GET /sse/stream
Authorization: Bearer <token>
```

**Response:** EventSource stream

---

#### 2. Get SSE Stats

```http
GET /sse/stats
Authorization: Bearer <token>
```

**Response:**

```json
{
  "success": true,
  "data": {
    "totalUsers": 50,
    "totalConnections": 50
  }
}
```

---

#### 3. Send Test Message

```http
POST /sse/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "message": "Test message"
}
```

---

### Realtime API Endpoints

#### 1. Notify Specific User

```http
POST /realtime/notify/:userId
Authorization: Bearer <token>
Content-Type: application/json

{
  "event": "notification",
  "data": {
    "title": "New Message",
    "body": "You have a new message"
  }
}
```

---

#### 2. Broadcast to All

```http
POST /realtime/broadcast
Authorization: Bearer <token>
Content-Type: application/json

{
  "event": "announcement",
  "data": {
    "message": "Server maintenance in 10 minutes"
  }
}
```

---

#### 3. Send to Channel

```http
POST /realtime/channel/:channel
Authorization: Bearer <token>
Content-Type: application/json

{
  "event": "message",
  "data": {
    "text": "Hello channel!"
  }
}
```

---

#### 4. Get Realtime Stats

```http
GET /realtime/stats
Authorization: Bearer <token>
```

**Response:**

```json
{
  "success": true,
  "data": {
    "sse": {
      "totalUsers": 50,
      "totalConnections": 50
    },
    "websocket": {
      "totalUsers": 150,
      "totalConnections": 200,
      "onlineUsers": ["user1", "user2", "user3"]
    }
  }
}
```

---

#### 5. Check User Online Status

```http
GET /realtime/online/:userId
Authorization: Bearer <token>
```

**Response:**

```json
{
  "success": true,
  "data": {
    "userId": "user123",
    "isOnline": true
  }
}
```

---

#### 6. Disconnect User (Admin)

```http
POST /realtime/disconnect/:userId
Authorization: Bearer <token>
Content-Type: application/json

{
  "reason": "Kicked by admin"
}
```

---

## 🌐 Horizontal Scaling

### How it Works

1. **Multiple Server Instances**
    - Run multiple instances of your app
    - Each instance connects to Redis
    - Load balancer distributes connections

2. **Redis PubSub**
    - SSE: Custom Redis PubSub implementation
    - WebSocket: Socket.IO Redis Adapter
    - Messages broadcast via Redis

3. **Example Flow**
   ```
   Client A → Server 1 → Redis Publish → Server 2 → Client B
   ```

### Setup Multiple Instances

**Server 1:**

```bash
PORT=3001 npm run dev
```

**Server 2:**

```bash
PORT=3002 npm run dev
```

**Nginx Load Balancer:**

```nginx
upstream websocket_backend {
    ip_hash; # Important for WebSocket
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
}
```

---

## 💡 Use Cases

### 1. Real-time Notifications

```typescript
// When order is created
await realtimeService.notifyUser(userId, 'order:created', {
    orderId: order.id,
    status: 'processing'
});
```

### 2. Chat Application

```typescript
// Send message to room
wsService.sendToRoom(roomId, 'chat:message', {
    userId,
    username,
    message,
    timestamp: new Date()
});
```

### 3. Live Dashboard Updates

```typescript
// Update stats for all admins
await sseService.sendToChannel('admin-dashboard', 'stats:update', {
    totalUsers: 1234,
    activeOrders: 56,
    revenue: 1000000
});
```

### 4. System Announcements

```typescript
// Broadcast to all users
await realtimeService.broadcastToAll('system:announcement', {
    type: 'maintenance',
    message: 'System will be down for 5 minutes',
    scheduledAt: '2026-01-18T10:00:00Z'
});
```

### 5. User Presence

```typescript
// Check if user is online
const isOnline = wsService.isUserOnline('user123');

// Get all online users
const stats = await wsService.getStats();
console.log(stats.onlineUsers);
```

---

## 🧪 Testing

### Test SSE with cURL

```bash
curl -N -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:3000/sse/stream
```

### Test WebSocket with wscat

```bash
npm install -g wscat

wscat -c "ws://localhost:3000" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Test with Postman

1. Create new WebSocket request
2. Add Authorization header
3. Connect and send events

---

## 📊 SSE vs WebSocket

| Feature             | SSE                  | WebSocket          |
|---------------------|----------------------|--------------------|
| **Direction**       | Unidirectional       | Bidirectional      |
| **Protocol**        | HTTP                 | WS/WSS             |
| **Browser Support** | Modern browsers      | All browsers       |
| **Reconnect**       | Automatic            | Manual (Socket.IO) |
| **Complexity**      | Simple               | Moderate           |
| **Use Case**        | Notifications, feeds | Chat, gaming       |
| **Overhead**        | Low                  | Low                |

### When to Use SSE?

- ✅ One-way communication (server → client)
- ✅ Simple notifications
- ✅ Live feeds
- ✅ Stock prices
- ✅ Dashboard updates

### When to Use WebSocket?

- ✅ Two-way communication
- ✅ Real-time chat
- ✅ Online gaming
- ✅ Collaborative editing
- ✅ Video conferencing

---

## 🔧 Configuration

### Environment Variables

```env
# Server
PORT=3000

# Redis (required for scaling)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0

# CORS
CORS_ORIGIN=*

# JWT (for WebSocket auth)
JWT_SECRET=your-secret-key
```

### Customize WebSocket Options

Edit `src/services/websocketService.ts`:

```typescript
this.io = new SocketIOServer(httpServer, {
    cors: {
        origin: process.env.CORS_ORIGIN || '*',
        methods: ['GET', 'POST'],
        credentials: true
    },
    transports: ['websocket', 'polling'],
    pingTimeout: 60000,     // Adjust timeout
    pingInterval: 25000,    // Adjust ping interval
});
```

---

## 🛠️ Troubleshooting

### SSE Connection Fails

1. Check JWT token validity
2. Check Redis connection
3. Check CORS settings
4. Check nginx buffering (disable with `X-Accel-Buffering: no`)

### WebSocket Connection Fails

1. Check JWT token in `auth.token`
2. Check Redis adapter connection
3. Check firewall/load balancer WebSocket support
4. Try polling transport first

### Messages Not Received on Other Servers

1. Check Redis PubSub connection
2. Check Redis host/port configuration
3. Check network between servers and Redis

### High Memory Usage

1. Check client connection count
2. Implement connection limits
3. Add timeout for inactive connections
4. Monitor Redis memory

---

## 📈 Performance Tips

1. **Use Redis Cluster** for high availability
2. **Enable compression** for large messages
3. **Implement rate limiting** to prevent abuse
4. **Use rooms/channels** instead of sending to individual users
5. **Close idle connections** after timeout
6. **Monitor metrics** (connections, messages/sec, latency)

---

## ✅ Summary

### What You Have Now:

- ✅ **SSE Service** dengan Redis PubSub scaling
- ✅ **WebSocket Service** dengan Socket.IO Redis Adapter
- ✅ **Unified Realtime Service** untuk kemudahan
- ✅ **JWT Authentication** untuk security
- ✅ **Horizontal Scaling** support
- ✅ **User tracking** dan presence
- ✅ **Rooms/Channels** support
- ✅ **Broadcasting** capabilities
- ✅ **Graceful shutdown** handling
- ✅ **Production-ready** architecture

### Architecture Benefits:

- 🚀 **Scalable**: Support multiple server instances
- 🔒 **Secure**: JWT authentication
- 💪 **Reliable**: Auto reconnect, fallback transports
- 📊 **Monitorable**: Stats and metrics
- 🎯 **Flexible**: Multiple targeting options
- 🧪 **Testable**: Easy to test and debug

**Your real-time infrastructure is ready for production!** 🎉
