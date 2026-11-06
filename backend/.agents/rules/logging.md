---
description: Mandatory structured logging using provided loggers across all backend templates, strictly prohibiting raw console and print statements
trigger: glob
globs: "**/*.ts"
---

# Backend Logging Standards & Rules

Semua kode backend (Express, FastAPI, Go Fiber, Spring Boot, dan Worker services) **WAJIB** menggunakan logger bawaan/terpusat yang telah disediakan oleh template. Dilarang keras menggunakan *raw console/print statement*.

---

## 1. DILARANG KERAS: Raw Print & Unstructured Logging

Dilarang menggunakan *standard output/error print statement* bawaan bahasa tanpa melalui wrapper logger yang disediakan:

- ❌ **Node.js / Express**:
  - Dilarang `console.log(...)`, `console.error(...)`, `console.warn(...)`, `console.info(...)`, `console.debug(...)`.
- ❌ **Python / FastAPI**:
  - Dilarang `print(...)` atau konfigurasi `logging.basicConfig()` ad-hoc.
- ❌ **Golang**:
  - Dilarang `fmt.Println(...)`, `fmt.Printf(...)`, `fmt.Print(...)`, atau standard library `log.Println(...)` / `log.Printf(...)`.
- ❌ **Java / Spring Boot**:
  - Dilarang `System.out.println(...)`, `System.err.println(...)`, atau `e.printStackTrace()`.

### Mengapa Dilarang?
1. **Kehilangan Konteks**: Raw print tidak menyertakan metadata penting seperti `requestId`, `userId`, `trace_id`, dan `span_id`.
2. **Merusak Log Aggregator**: Log collector (Datadog, Grafana Loki, AWS CloudWatch, ELK) membutuhkan format **JSON terstruktur** per baris. Raw print merusak parsing dan pencarian log.
3. **Tidak Mendukung Log Level**: Raw print tidak bisa difilter di level produksi (`DEBUG`, `INFO`, `WARN`, `ERROR`).
4. **Bypass File Rotation & Tracing**: Raw print tidak masuk ke sistem rotasi file log harian maupun integrasi OpenTelemetry.
5. **Kebocoran Data Sensitif (Security Risk)**: Raw print rawan membocorkan data sensitif tanpa sanitasi.

---

## 2. Wajib Menggunakan Logger Bawaan Sesuai Template

Setiap template backend telah memiliki konfigurasi logger berbasis JSON dengan injeksi konteks otomatis:

### A. Express / TypeScript Backend (`template-express`)
Gunakan logger singleton dari `src/utils/logger.ts`:

```typescript
// ❌ SALAH
console.log('User created:', user.id);
console.error('Database query failed:', err);

// ✅ BENAR
import logger from '@/utils/logger';

// Info dengan context object
logger.info('User created successfully', { userId: user.id });

// Error dengan error object & metadata
logger.error('Failed to create user', { error: err, email: req.body.email });

// Warning
logger.warn('Token nearing expiration', { userId: user.id });
```
*Catatan*: `requestId`, `userId`, `trace_id`, dan `span_id` otomatis diinjeksikan ke setiap baris log melalui Winston format & AsyncLocalStorage.

---

### B. FastAPI / Python Backend (`template-fastapi-async`)
Gunakan standard Python `logging` yang telah dikonfigurasi melalui `lib/logger.py`:

```python
# ❌ SALAH
print(f"Processing order: {order_id}")
print(f"Error: {e}")

# ✅ BENAR
import logging

logger = logging.getLogger(__name__)

# Info dengan parameter atau extra fields
logger.info("Order processed successfully for order_id: %s", order_id, extra={"user_id": user_id})

# Error dengan exception trace lengkap (exc_info=True)
logger.error("Failed to process payment for order_id: %s", order_id, exc_info=True)

# Warning
logger.warn("Cache miss for key: %s, falling back to database", cache_key)
```
*Catatan*: `ContextFilter` otomatis menambahkan `request_id`, `user_id`, `trace_id`, dan `span_id` dari contextvars ke output JSON.

---

### C. Golang / Fiber Backend (`template-golang`)
Gunakan package `pkg/logging` berbasis `log/slog`:

```go
// ❌ SALAH
fmt.Println("Starting background worker...")
fmt.Printf("Error processing message: %v\n", err)

// ✅ BENAR
import (
    "log/slog"
    "your-project/pkg/logging"
)

// Logging umum dengan slog attributes
logging.Info("worker started successfully",
    slog.String("queue", queueName),
    slog.Int("concurrency", workersCount),
)

// Logging error dengan helper logging.Err()
logging.Error("failed to process message",
    logging.Err(err),
    slog.String("message_id", msgID),
)

// Di dalam handler/request context, gunakan request-scoped logger:
logger := logging.LoggerFromFiber(c) // atau logging.LoggerFromContext(ctx)
logger.Info("user profile fetched", slog.String("user_id", userID))
```

---

### D. Spring Boot / Java Backend (`template-java`)
Gunakan Lombok `@Slf4j` atau `LoggerFactory.getLogger(...)`:

```java
// ❌ SALAH
System.out.println("User login: " + username);
e.printStackTrace();

// ✅ BENAR
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

@Slf4j
@Service
public class UserService {

    public void processLogin(String email) {
        // Gunakan placeholder {}, JANGAN string concatenation (+)
        log.info("User login attempt for email: {}", email);
    }

    public void handleError(String userId, Exception ex) {
        // Oper exception sebagai argumen terakhir untuk stack trace otomatis
        log.error("Failed to update user profile for userId: {}", userId, ex);
    }
}
```
*Catatan*: `RequestIdFilter` otomatis mengisi MDC sehingga `requestId` tercetak di setiap baris JSON log melalui `logback-spring.xml`.

---

### E. Worker Templates (Go, Python, TypeScript, Java)
Pada modul worker/queue consumer:
- **TypeScript Worker**: Gunakan logger terstruktur dari modul utilitas worker, bukan `console.log`.
- **Python Worker**: Gunakan `logging.getLogger(__name__)` dengan format JSON, bukan `print()`.
- **Go Worker**: Gunakan `pkg/logging` atau `slog.Default()`, bukan `fmt.Println()`.
- **Java Worker**: Gunakan `@Slf4j`, bukan `System.out.println()`.

---

## 3. Standar Pemilihan Log Level

Pilih log level secara tepat sesuai fungsi dan peristiwanya:

| Level | Kapan Digunakan | Contoh Skenario |
| :--- | :--- | :--- |
| **`DEBUG`** | Detail diagnostik rendah untuk investigasi bug saat development. | Payload mentah internal, trace langkah algoritma, detail cache lookup. |
| **`INFO`** | Kejadian normal yang menandakan alur bisnis berjalan semestinya. | Server started, endpoint dipanggil, user terdaftar, worker selesai memproses task. |
| **`WARN`** | Anomali atau isu yang dapat di-handle/dipulihkan (*recoverable*). | Retry connection, cache miss fallback ke DB, rate limit mendekati ambang batas, token refresh hampir expired. |
| **`ERROR`** | Kesalahan sistem atau kegagalan transaksi yang membutuhkan perhatian. | Database query fail, unhandled exception, third-party API timeout/5xx, task worker gagal. |

---

## 4. Keamanan & Larangan Logging Data Sensitif (PII / Secrets)

**DILARANG KERAS** mencantumkan data sensitif pada parameter atau pesan log:
- ❌ **Password & PIN** (termasuk *hashed password*).
- ❌ **Token Otentikasi**: Raw JWT `access_token`, `refresh_token`, API Secret Keys, Basic Auth headers.
- ❌ **Informasi Finansial / PII**: CVV/CVC, nomor kartu kredit lengkap, NIK, nomor rekening sensitif.

Gunakan masking jika field tersebut harus di-log (misal: `email.replace(/(?<=.{2}).(?=.*@)/g, '*')` atau `card.slice(-4)`).

---

## 5. Checklist Verifikasi (Definition of Done)

Sebelum menyelesaikan task backend:
1. [ ] Tidak ada baris `console.log`, `print()`, `fmt.Println`, atau `System.out.println`.
2. [ ] Semua log menggunakan instance logger yang disediakan template.
3. [ ] Pemilihan level log sudah tepat (`DEBUG`, `INFO`, `WARN`, `ERROR`).
4. [ ] Pesan log tidak menggabungkan string manual jika logging framework mendukung structured metadata / `{}` placeholders.
5. [ ] Tidak ada password, token, atau informasi rahasia yang tercetak di log.
