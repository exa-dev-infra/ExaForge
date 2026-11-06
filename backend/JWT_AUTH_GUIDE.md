# JWT Authentication Setup - Complete!

## ✅ What's Been Implemented

### 1. **JWT Service** (`src/services/jwtService.ts`)

- Generate access tokens (15 minutes expiry)
- Generate refresh tokens (7 days expiry)
- Verify tokens
- Decode tokens
- Get token expiry

### 2. **Password Service** (`src/services/passwordService.ts`)

- Hash passwords with bcrypt
- Compare passwords
- Validate password strength

### 3. **Authentication Middleware** (`src/middleware/auth.ts`)

- `authenticate` - Verify access token
- `optionalAuth` - Optional authentication
- `authorize` - Check user roles
- `checkOwnership` - Verify resource ownership
- `rateLimitByUser` - User-based rate limiting (placeholder)

### 4. **Auth Routes** (`src/routes/auth.ts`)

- `POST /auth/register` - Register new user
- `POST /auth/login` - Login user
- `POST /auth/refresh` - Refresh access token
- `POST /auth/logout` - Logout user
- `POST /auth/change-password` - Change password (protected)
- `GET /auth/me` - Get current user (protected)

### 5. **Database Migration**

- Added `password`, `role`, `last_login` columns to users table

### 6. **Environment Variables**

- `JWT_ACCESS_SECRET` - Secret for access tokens
- `JWT_REFRESH_SECRET` - Secret for refresh tokens
- `JWT_ACCESS_EXPIRY` - Access token expiry (default: 15m)
- `JWT_REFRESH_EXPIRY` - Refresh token expiry (default: 7d)

## 🚀 Quick Start

### 1. Run Migration

```bash
npm run migrate:up
```

### 2. Start Server

```bash
npm run dev
```

### 3. Test Authentication Flow

#### Register User

```bash
POST /auth/register
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "role": "user"
}
```

Response:

```json
{
  "success": true,
  "message": "User registered successfully",
  "data": {
    "user": {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com",
      "role": "user"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJIUzI1NiIs...",
      "refreshToken": "eyJhbGciOiJIUzI1NiIs..."
    }
  },
  "code": "CREATED",
  "timestamp": "2026-01-17..."
}
```

#### Login

```bash
POST /auth/login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

#### Access Protected Route

```bash
GET /auth/me
Authorization: Bearer <access_token>
```

#### Refresh Token

```bash
POST /auth/refresh
Content-Type: application/json

{
  "refreshToken": "eyJhbGciOiJIUzI1NiIs..."
}
```

#### Logout

```bash
POST /auth/logout
Content-Type: application/json

{
  "refreshToken": "eyJhbGciOiJIUzI1NiIs..."
}
```

## 🔒 Using Authentication Middleware

### Protect Single Route

```typescript
import { authenticate } from '../middleware/auth';

router.get('/profile', authenticate, async (req, res) => {
    // req.user is available here
    const userId = req.user?.userId;
});
```

### Protect Multiple Routes

```typescript
// In app.ts
app.use('/api/protected', authenticate);
```

### Check Role Authorization

```typescript
import { authenticate, authorize } from '../middleware/auth';

// Only admin can access
router.delete('/users/:id', authenticate, authorize('admin'), async (req, res) => {
    // Delete user logic
});

// Admin or moderator can access
router.post('/moderate', authenticate, authorize('admin', 'moderator'), async (req, res) => {
    // Moderation logic
});
```

### Check Resource Ownership

```typescript
import { authenticate, checkOwnership } from '../middleware/auth';

// User can only update their own profile
router.put('/users/:id', authenticate, checkOwnership('id'), async (req, res) => {
    // Update user logic
});
```

### Optional Authentication

```typescript
import { optionalAuth } from '../middleware/auth';

// Public route but can show different content for logged-in users
router.get('/products', optionalAuth, async (req, res) => {
    if (req.user) {
        // Show personalized products
    } else {
        // Show public products
    }
});
```

## 📝 Example Protected Routes

```typescript
import { Router } from 'express';
import { authenticate, authorize, checkOwnership } from '../middleware/auth';
import ErrorHandler from '../middleware/errorHandler';
import { ApiResponse } from '../types/response';

const router = Router();

// Protected route - requires authentication
router.get('/dashboard', authenticate, ErrorHandler.catchAsync(async (req, res) => {
    const userId = req.user?.userId;
    
    // Get user dashboard data
    const response = ApiResponse.success({ userId }, 'Dashboard data');
    res.json(response);
}));

// Admin only route
router.get('/admin/users', authenticate, authorize('admin'), ErrorHandler.catchAsync(async (req, res) => {
    // Get all users (admin only)
    const response = ApiResponse.success([], 'All users');
    res.json(response);
}));

// User can only access their own data
router.get('/profile/:userId', authenticate, checkOwnership('userId'), ErrorHandler.catchAsync(async (req, res) => {
    // Get user profile
    const response = ApiResponse.success({}, 'User profile');
    res.json(response);
}));

export default router;
```

## 🔐 Token Flow

### Registration/Login Flow

1. User registers or logs in
2. Server generates access token (15 min) and refresh token (7 days)
3. Refresh token is stored in Redis
4. Both tokens are returned to client
5. Client stores tokens (access token in memory, refresh token in httpOnly cookie or secure storage)

### Authentication Flow

1. Client sends request with `Authorization: Bearer <access_token>`
2. Middleware verifies access token
3. If valid, `req.user` is populated
4. Route handler can access user data

### Token Refresh Flow

1. When access token expires, client sends refresh token
2. Server verifies refresh token
3. Server checks if refresh token exists in Redis
4. If valid, generates new token pair
5. Old refresh token is replaced in Redis
6. New tokens are returned to client

### Logout Flow

1. Client sends refresh token
2. Server removes refresh token from Redis
3. Client should delete all stored tokens

## 🛡️ Security Features

✅ **Password Hashing** - bcrypt with 10 rounds
✅ **Password Strength Validation** - Min 8 chars, uppercase, lowercase, number, special char
✅ **Token Expiry** - Short-lived access tokens (15 min)
✅ **Refresh Token Rotation** - New refresh token on each refresh
✅ **Token Storage in Redis** - Refresh tokens stored server-side
✅ **Token Invalidation** - Tokens can be revoked
✅ **Role-Based Access Control** - Authorization middleware
✅ **Resource Ownership Check** - Users can only access their own resources

## 📊 Database Schema

### Users Table (Updated)

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    last_login TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
```

### Redis Keys

```
refresh_token:{userId} - Stores refresh token for user
```

## 🧪 Testing with cURL

```bash
# Register
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"John","email":"john@example.com","password":"SecurePass123!"}'

# Login
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"john@example.com","password":"SecurePass123!"}'

# Get profile (protected)
curl http://localhost:3000/auth/me \
  -H "Authorization: Bearer <access_token>"

# Refresh token
curl -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refreshToken":"<refresh_token>"}'

# Logout
curl -X POST http://localhost:3000/auth/logout \
  -H "Content-Type: application/json" \
  -d '{"refreshToken":"<refresh_token>"}'
```

## 🎯 Password Requirements

- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character (!@#$%^&*(),.?":{}|<>)

## 📚 Files Created/Modified

```
src/
├── services/
│   ├── jwtService.ts          ✅ NEW - JWT token management
│   └── passwordService.ts     ✅ NEW - Password hashing & validation
├── middleware/
│   └── auth.ts                ✅ NEW - Authentication middleware
├── routes/
│   └── auth.ts                ✅ NEW - Auth endpoints
└── app.ts                     ✅ UPDATED - Added auth routes

migrations/
└── *_add-auth-fields-to-users.ts  ✅ NEW - Add password, role columns

.env                           ✅ UPDATED - Added JWT secrets
```

## ⚠️ Security Notes

1. **Change JWT Secrets** - Update `JWT_ACCESS_SECRET` and `JWT_REFRESH_SECRET` in production
2. **Use HTTPS** - Always use HTTPS in production
3. **Secure Token Storage** - Store access tokens in memory, refresh tokens in httpOnly cookies
4. **Rate Limiting** - Implement rate limiting on auth endpoints
5. **CORS Configuration** - Configure CORS properly
6. **Audit Logging** - Log all authentication attempts

## 🎉 Status

- **JWT Service**: ✅ IMPLEMENTED
- **Password Service**: ✅ IMPLEMENTED
- **Auth Middleware**: ✅ IMPLEMENTED
- **Auth Routes**: ✅ IMPLEMENTED
- **Database Migration**: ✅ CREATED
- **Documentation**: ✅ COMPLETE
- **Build**: ✅ SUCCESS

---

**Ready to use!** JWT authentication with access/refresh token flow is fully implemented!
