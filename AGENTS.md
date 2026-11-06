# Antigravity Agent Guidelines & Task Management Rules

Dokumen ini berisi pedoman permanen untuk AI Coding Assistant saat berinteraksi dan mengelola repository **ExaForge** (Lightweight Self-Hosted CI/CD Engine).

---

## 📌 Aturan Task Management (MANDATORY)

Setiap agen yang bekerja pada repository ini **WAJIB** mematuhi siklus task management berikut secara mandiri tanpa perlu diingatkan oleh pengguna:

1. **Target Project ID**:
   - Target Project ID resmi untuk repository **ExaForge** di sistem Task Management adalah **Project ID: 5** (`name: Template` / ExaForge CI/CD Engine).
   - Setiap pembuatan task baru (`create_task`), **WAJIB menyertakan `project_id: 5`** serta tags `["exaforge", "cicd", "controller", "agent", "rust"]`.

2. **Pencarian & Pembuatan Task Aktif**:
   - Setiap kali user meminta penambahan fitur, perbaikan bug, atau enhancement baru: **selalu cari task aktif terlebih dahulu** menggunakan tool `list_tasks` (filter `project_id: 5`, status `in_progress` / `todo` atau keyword pencarian).
   - Jika task untuk pekerjaan tersebut belum ada, **otomatis buatkan task baru** (`create_task`) dengan status `in_progress`, cantumkan deskripsi jelas, dan pecah menjadi checklist `subtasks`.

3. **Penyelesaian & Logging Progres**:
   - Setelah pekerjaan selesai diimplementasikan, diverifikasi (`cargo check` / `cargo test` lulus), dan siap di-commit/push, **otomatis perbarui progress dan status task**:
     - Gunakan `log_task_progress` untuk mencatat catatan penyelesaian.
     - Update subtask yang selesai (`completed_subtask_ids`) dan ubah status task utama menjadi `done` (`update_task_status`).

4. **Eksekusi Mandiri**:
   - Jalankan seluruh siklus task management ini (Cari -> Buat jika belum ada -> Kerjakan -> Verifikasi -> Mark Done) secara otonom di setiap giliran kerja tanpa perlu konfirmasi manual dari user untuk sekadar update task.

---

## 🏗️ Standar Arsitektur & Teknologi Repo (@exa31/create-stack template-rust)

Repository ini disusun menggunakan **100% Pure Rust** dengan pola arsitektur **Controller + Worker Agent** tanpa frontend SPA terpisah:

### 1. Unified Single Binary / Dual-Mode CLI
- **`exaforge serve`**: Menjalankan Controller (HTTP Server Axum, Web Dashboard SSR via Askama, REST API, dan WebSocket Hub).
- **`exaforge agent`**: Menjalankan Worker Agent daemon (koneksi outbound ke Controller via WebSocket, isolasi OS Process Group, incremental git cache, container build, log streaming baris demi baris dengan secret masking).

### 2. Frontend / UI: Embedded Server-Side Rendering (Askama)
- Menggunakan **Askama Template Engine** (`askama` + `askama_axum`) untuk rendering HTML langsung di server (*zero frontend build steps / no node_modules*).
- Tampilan modern Dark Mode menggunakan Tailwind CSS (CDN / embedded) + Alpine.js untuk interaktivitas real-time (live terminal log stream via WebSocket, status badge polling, modal input trigger).
- Semua template disimpan di `templates/` (misal `base.html`, `dashboard.html`, `project.html`, `service.html`, `pipeline.html`, `secrets.html`).

### 3. Backend & Core Engine (Axum + Tokio + SQLx)
- **Runtime**: Tokio async runtime.
- **Web Framework**: Axum 0.7 (`ws`, `macros`, `multipart`, `form`) + Tower HTTP + Axum Extra.
- **Database**: PostgreSQL dengan SQLx pool dan migrasi SQL terkompilasi (`migrations/`).
- **Database Transactions**:
  - Semua operasi mutasi (`INSERT`, `UPDATE`, `DELETE`) **WAJIB** berada dalam transaksi (`pool.begin().await?`).
  - Pure parameter passing: `&mut sqlx::Transaction<'_, sqlx::Postgres>`.
- **Security & Kriptografi**:
  - Enkripsi Secret at-rest: `AES-256-GCM` (32-byte master key, 12-byte IV unik, 16-byte auth tag).
  - Autentikasi Pengguna: Argon2 password hashing + JWT session cookie / Bearer token.
  - Webhook Git: HMAC SHA-256 verification.
- **Process Group Isolation (Agent Engine)**:
  - Setiap step command dieksekusi di dalam Process Group tersendiri (`nix::unistd::setpgid` / `libc::setsid`).
  - Saat timeout atau menerima sinyal pembatalan, Agent mengirim `SIGTERM` / `SIGKILL` ke seluruh process group (`-pgid`) untuk mencegah orphan/zombie process.

### 4. Logging & Error Handling
- ❌ **DILARANG KERAS** menggunakan raw print: `println!`, `eprintln!`, `dbg!`.
- ✅ **WAJIB** menggunakan `tracing::{info, warn, error, debug}`.
- Semantic error codes didefinisikan secara terpusat di `src/constants/codes.rs`.
