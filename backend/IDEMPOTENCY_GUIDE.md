# 🔒 Idempotency Key Guide (Express.js / TypeScript)

Panduan implementasi dan penggunaan fitur **Idempotency Key** berbasis standar IETF (*draft-ietf-httpapi-idempotency-key-header*) dengan distributed Redis lock dan in-memory fallback di Express.

---

## 📌 1. Cara Kerja (Transparent Express Middleware)

Middleware idempotency dipasang di `src/app.ts` pada router API:
1. Client menyertakan header `Idempotency-Key: <uuid>` (atau `X-Idempotency-Key`) pada request mutating (`POST`, `PATCH`).
2. Middleware menghitung SHA-256 fingerprint dari:
   ```
   {userId}:{HTTP_METHOD}:{requestPath}:{requestBody}
   ```
3. Middleware memeriksa storage:
   - **Key belum ada**: Mengambil distributed lock via Redis `SET ... NX EX` dengan status `in_progress`.
   - **Key sedang diproses (in_progress)**: Mengembalikan `409 Conflict` (`IDEMPOTENCY_CONFLICT`).
   - **Key sudah selesai (completed) tapi payload berbeda**: Mengembalikan `422 Unprocessable Entity` (`IDEMPOTENCY_PAYLOAD_MISMATCH`).
   - **Key sudah selesai (completed) dan payload sama**: Mengembalikan cached response langsung dari storage beserta header:
     ```http
     Idempotent-Replayed: true
     X-Idempotency-Replayed: true
     ```
     Handler controller **tidak akan dieksekusi ulang**.
4. **Error handling (5xx)**: Jika request gagal dengan status code 5xx, lock otomatis dihapus (`delete`) agar client dapat melakukan retry.

---

## 🚀 2. Contoh Penggunaan (cURL Examples)

Gunakan endpoint mutation seperti `POST /auth/register` atau `POST /uploads`:

### A. Request Pertama (Eksekusi Baru)
```bash
curl -i -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: exp-uuid-112233" \
  -d '{"name":"John Doe","email":"john@example.com","password":"Password123!"}'
```
**Response (201 Created / 200 OK)**:
```json
{
  "success": true,
  "code": "SUCCESS",
  "message": "User registered successfully",
  "data": { ... }
}
```

---

### B. Request Ulang dengan Key yang Sama (Replayed Response)
Kirim kembali request yang sama persis:
```bash
curl -i -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: exp-uuid-112233" \
  -d '{"name":"John Doe","email":"john@example.com","password":"Password123!"}'
```
**Response Header**:
```http
HTTP/1.1 200 OK
Idempotent-Replayed: true
X-Idempotency-Replayed: true
```
> **Catatan**: Handler registrasi tidak dieksekusi ulang, response yang dikembalikan diambil langsung dari cache storage.

---

### C. Request dengan Key Sama tapi Payload Berbeda (Payload Mismatch)
```bash
curl -i -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: exp-uuid-112233" \
  -d '{"name":"Jane Doe","email":"jane@example.com","password":"DifferentPassword123!"}'
```
**Response (422 Unprocessable Entity)**:
```json
{
  "success": false,
  "code": "IDEMPOTENCY_PAYLOAD_MISMATCH",
  "message": "Idempotency key payload does not match the original request",
  "data": null
}
```

---

## 🔌 3. Arsitektur Koneksi Redis (Connection Pooling)

Aplikasi Express menggunakan **Shared Singleton Redis Client**:
- Instance `RedisClient.getInstance()` (node-redis v4) dibuat sebagai singleton di `src/lib/redis.ts`.
- Di `src/app.ts`, instance yang sama disuntikkan ke `new RedisIdempotencyStorage(RedisClient.getInstance())`.
- Koneksi TCP ke Redis dipakai bersama oleh seluruh fitur (caching, auth token blacklist, dan idempotency key).

---

## ⚙️ 4. Konfigurasi Environment Variables

| Variabel | Default | Keterangan |
| :--- | :--- | :--- |
| `IDEMPOTENCY_ENABLED` | `true` | Mengaktifkan/menonaktifkan middleware idempotency |
| `IDEMPOTENCY_LIFETIME` | `86400` | TTL masa simpan response yang telah selesai (detik, default 24 jam) |
| `IDEMPOTENCY_TIMEOUT` | `30` | Durasi lock in-flight request sebelum kedaluwarsa (detik) |
| `IDEMPOTENCY_STORAGE` | `redis` | Jenis storage: `redis` atau `memory` |
