# 🦀 Rust Axum + SQLx Backend Starter Kit

Production-grade, asynchronous REST API starter kit built with **Rust**, **Axum 0.7**, and **SQLx (PostgreSQL)**, adhering to strict monorepo Clean Architecture guidelines, structured JSON logging, semantic error handling, and transactional database safety.

> 📖 **Baru belajar Rust?** Baca panduan kilat sintaks, keyword (`pub`, `async`, lifetime `'a`), smart pointers (`Arc`, `Box`), dan bedah baris kode di:  
> 👉 [**docs/RUST_LEARNING_GUIDE.md**](docs/RUST_LEARNING_GUIDE.md)

---

## 🚀 Fitur Utama

- **Web Framework**: [Axum 0.7](https://github.com/tokio-rs/axum) yang dibangun di atas ekosistem Tokio & Tower.
- **Database & Migration**: [SQLx 0.8](https://github.com/launchbadge/sqlx) (PostgreSQL) dengan pure parameter passing dan migrasi SQL otomatis.
- **Structured Logging**: `tracing` + `tracing-subscriber` dengan output JSON terstruktur per baris (Zero raw `println!`).
- **Input Validation**: `validator` crate dengan custom Axum extractor (`ValidatedJson`).
- **Semantic Error Handling**: Centralized error codes (`USER_NOT_FOUND`, `INVALID_CREDENTIALS`, etc.) dan format envelope API standar (`BaseResponse<T>`).
- **Complete Auth & Refresh Token Flow**:
  - **SHA-256 Token Hashing**: Token refresh disimpan dalam bentuk hash digest di database.
  - **Token Reuse & Theft Detection**: Otomatis mendeteksi pemakaian ulang token lama dan mencabut seluruh sesi user demi keamanan.
  - **Multi-session Management**: Manajemen sesi aktif per user-agent/IP, `logout`, `logout-all`, dan `change-password`.
- **OpenTelemetry (OTEL)**: OTLP distributed tracing exporter via Tonic gRPC dengan W3C trace context propagation (terintegrasi dengan `tracing`).
- **Distributed Realtime Bridge (Redis / RabbitMQ)**: Mendistribusikan event WebSocket & SSE ke seluruh replika container via Redis Pub/Sub (`realtime:broadcast`) atau RabbitMQ Fanout Exchange (`realtime.events`), dengan graceful fallback ke in-memory `local` mode.
- **Environment Parity**: Selaras 100% dengan parameter konfigurasi backend monorepo (`template-golang`, `template-express`), mendukung standalone URL maupun granular host/port/credentials.
- **Quality Gates**: `make check` (fmt-check, clippy linter, dan unit test suite).

---

## 📁 Struktur Direktori

```text
src/
├── main.rs                   # Entrypoint server & graceful shutdown
├── lib.rs                    # Router builder & app assembly
├── config/                   # App configuration & environment loader
├── constants/                # Semantic error & response codes
├── database/                 # PostgreSQL connection pool & migrations
├── pkg/                      # Reusable internal packages
│   ├── auth/                 # JWT signing/verification & Argon2 hashing
│   ├── errors/               # Centralized AppError & HTTP conversion
│   ├── logging/              # Tracing JSON logger & Tower trace layer
│   ├── realtime/             # Broadcast hub for WS & SSE
│   ├── response/             # Standard BaseResponse & PaginatedResponse
│   └── validation/           # ValidatedJson extractor
└── modules/                  # Feature modules (Clean Architecture)
    ├── auth/                 # DTO, repository, service, handler
    ├── user/                 # Profile, list, and user management
    └── realtime/             # WebSocket and SSE streaming handlers
```

---

## ⚡ Quickstart

### 1. Prasyarat
- Rust toolchain (1.80+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- PostgreSQL 16+ (atau gunakan `docker-compose up -d postgres`)

### 2. Setup Environment
```bash
cp .env.example .env
```

### 3. Menjalankan Server
```bash
# Jalankan langsung
cargo run
# atau via make
make run
```

Server akan aktif di `http://localhost:8080`.

---

## 🗄️ Database Migrations & Rollback

Template ini mendukung migrasi bolak-balik (**Reversible Migrations**: `.up.sql` dan `.down.sql`) dengan SQLx:

```bash
# Jalankan seluruh migrasi yang belum diaplikasikan (Up)
make migrate-up

# Rollback 1 langkah migrasi terakhir (Down)
make migrate-down

# Buat migrasi baru berpasangan (.up.sql & .down.sql)
make migrate-create name=add_items_table

# Cek status migrasi yang telah diterapkan di database
make migrate-status
```

---

## 🔐 API Endpoints Reference

### 1. Root & Health Probes
| Method | Endpoint | Proteksi | Deskripsi |
|---|---|---|---|
| `GET` | `/` | Publik | Welcome response dan server time |
| `GET` | `/health` | Publik | Status komprehensif DB, Redis, RabbitMQ, Realtime, dan feature flags |

### 2. Auth & Security Endpoints (`/api/v1/auth`)
| Method | Endpoint | Proteksi | Deskripsi |
|---|---|---|---|
| `POST` | `/api/v1/auth/register` | Publik | Registrasi akun baru & set refresh token cookie |
| `POST` | `/api/v1/auth/login` | Publik | Login & set HTTP-only refresh token cookie |
| `POST` | `/api/v1/auth/refresh` | Cookie/Body | Rotasi token baru + deteksi pencurian token lama |
| `POST` | `/api/v1/auth/logout` | Cookie/Body | Revokasi sesi aktif saat ini & bersihkan cookie |
| `POST` | `/api/v1/auth/logout-all` | Bearer Token | Revokasi seluruh sesi login di semua device |
| `POST` | `/api/v1/auth/change-password` | Bearer Token | Ubah password & otomatis revokasi seluruh sesi lama |
| `GET` | `/api/v1/auth/sessions` | Bearer Token | Tampilkan riwayat device/sesi yang aktif |
| `GET` | `/api/v1/auth/me` | Bearer Token | Dapatkan ringkasan profil user saat ini |

### 3. User Self-Service & Admin CRUD (`/api/v1/users`)
| Method | Endpoint | Proteksi | Deskripsi |
|---|---|---|---|
| `GET` | `/api/v1/users/profile` | Bearer Token | Ambil profil pengguna yang sedang login |
| `PUT` | `/api/v1/users/profile` | Bearer Token | Update profil pengguna (nama, telepon) |
| `GET` | `/api/v1/users` | Admin Only | List pengguna (paginasi `page`, `limit`, filter `search`, `role`, `is_active`) |
| `GET` | `/api/v1/users/:id` | Admin Only | Ambil detail pengguna berdasarkan ID |
| `POST` | `/api/v1/users` | Admin Only | Buat pengguna baru oleh admin (dengan transaksi & hashing) |
| `PUT` | `/api/v1/users/:id` | Admin Only | Update data pengguna (nama, email, telepon, role, status aktif) |
| `DELETE` | `/api/v1/users/:id` | Admin Only | Soft delete pengguna (`is_active = false` & cabut seluruh sesi) |
| `DELETE` | `/api/v1/users/:id/hard` | Admin Only | Hard delete (hapus permanen dari database) |

### 4. Realtime Streaming & Examples (`/api/v1/realtime`)
| Method | Endpoint | Proteksi | Deskripsi |
|---|---|---|---|
| `GET` | `/api/v1/realtime/ws` | Publik/Client | WebSocket duplex connection hub |
| `GET` | `/api/v1/realtime/sse` | Publik/Client | Server-Sent Events downstream event stream |
| `POST` | `/api/v1/realtime/broadcast` | Publik/Auth | Menerbitkan pesan broadcast ke seluruh client lokal & replika bridge |

### 5. Protected & Optional Example Endpoints
| Method | Endpoint | Proteksi | Deskripsi |
|---|---|---|---|
| `GET` | `/api/v1/protected/data` | Bearer Token | Contoh endpoint proteksi standar (`AuthenticatedUser`) |
| `GET` | `/api/v1/public-or-private` | Opsional | Contoh endpoint fleksibel (`OptionalAuthUser`) |

---

## 🔁 Idempotency Key Handling
Mendukung header `Idempotency-Key` atau `X-Idempotency-Key` pada mutasi `POST`, `PUT`, dan `PATCH`:
- Menghindari duplikasi request pada retry jaringan.
- Jika request sedang diproses, otomatis return `409 Conflict` (`IDEMPOTENCY_IN_FLIGHT`).
- Jika request telah selesai, otomatis me-replay response asli beserta header `Idempotent-Replayed: true`.

---

## 📡 OpenTelemetry (OTEL) Distributed Tracing

Template ini siap dihubungkan ke OpenTelemetry Collector / Jaeger / Grafana Tempo:

1. Di file `.env`, atur:
   ```env
   OTEL_ENABLED=true
   OTEL_SERVICE_NAME=my-rust-service
   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
   ```
2. Seluruh trace request Axum, span HTTP, dan log event otomatis diekspor melalui gRPC Tonic ke OTLP endpoint dengan format W3C trace context.

---

## 🌐 Distributed Realtime Bridge

Saat aplikasi di-scale menjadi multi-container / multi-pod, WebSocket dan SSE didistribusikan antar replika melalui broker:

| Bridge Mode (`REALTIME_BRIDGE`) | Deskripsi |
|---|---|
| `redis` (Default) | Memanfaatkan Redis Pub/Sub (`realtime:broadcast`) untuk fan-out antar instance. |
| `rabbitmq` | Memanfaatkan RabbitMQ Fanout Exchange (`realtime.events`) dan antrean eksklusif per replika. |
| `both` | Mempublikasikan ke Redis dan RabbitMQ secara simultan. |
| `local` | In-memory broadcast (tokio channel) untuk mode offline / standalone development tanpa broker. |

> **Catatan Resiliensi**: Jika broker yang dipilih tidak dapat dijangkau saat startup, aplikasi **tidak akan crash**, melainkan otomatis mendegradasi ke mode `local` dan mencatat warning terstruktur.

---

## ⚙️ Environment Variables Reference

Konfigurasi `.env` selaras penuh dengan template backend lain di monorepo (`template-golang`):

| Variable | Default | Deskripsi |
|---|---|---|
| `APP_NAME` / `APP_ENV` | `template-rust` / `development` | Nama dan environment aplikasi |
| `SERVER_HOST` / `SERVER_PORT` | `0.0.0.0` / `8080` | Host dan port listener Axum |
| `LOG_LEVEL` | `info` | Level log (`trace`, `debug`, `info`, `warn`, `error`) |
| `DATABASE_URL` | Postgre connection string | Jika kosong, disusun otomatis dari `DB_HOST`, `DB_PORT`, dsb. |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Jika kosong, disusun otomatis dari `REDIS_HOST`, `REDIS_PORT`, dsb. |
| `RABBITMQ_URL` | `amqp://guest:guest@localhost:5672/` | Jika kosong, disusun otomatis dari `RABBITMQ_*` |
| `REALTIME_BRIDGE` | `redis` | Broker realtime: `redis`, `rabbitmq`, `both`, atau `local` |
| `JWT_ACCESS_SECRET` | *secret-key* | Kunci rahasia untuk HMAC sign access token |
| `JWT_REFRESH_SECRET` | *secret-key* | Kunci rahasia untuk HMAC sign refresh token |
| `OTEL_ENABLED` | `false` | Mengaktifkan/menonaktifkan OpenTelemetry tracing |
| `OTEL_EXPORTER_OTLP_ENDPOINT`| `http://localhost:4317` | Alamat OTLP gRPC endpoint |

---

## 🧪 Testing & Quality Gates

```bash
# Jalankan seluruh quality gate (fmt, clippy, test)
make check

# Jalankan test suite saja
make test

# Format kode
make fmt
```

---

## 🐳 Docker Deployment

```bash
# Build dan jalankan dengan Docker Compose
docker-compose up --build
```

