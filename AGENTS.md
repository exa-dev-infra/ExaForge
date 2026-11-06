# Antigravity Agent Guidelines & Task Management Rules

Dokumen ini berisi pedoman permanen untuk AI Coding Assistant saat berinteraksi dan mengelola repository **ExaForge** (Lightweight Self-Hosted CI/CD Engine).

---

## 📌 Aturan Task Management (MANDATORY)

Setiap agen yang bekerja pada repository ini **WAJIB** mematuhi siklus task management berikut secara mandiri tanpa perlu diingatkan oleh pengguna:

1. **Target Project ID**:
   - Target Project ID resmi untuk repository **ExaForge** di sistem Task Management adalah **Project ID: 5** (`name: Template` / ExaForge CI/CD Engine).
   - Setiap pembuatan task baru (`create_task`), **WAJIB menyertakan `project_id: 5`** serta tags `["exaforge", "cicd", "controller", "agent"]`.

2. **Pencarian & Pembuatan Task Aktif**:
   - Setiap kali user meminta penambahan fitur, perbaikan bug, atau enhancement baru: **selalu cari task aktif terlebih dahulu** menggunakan tool `list_tasks` (filter `project_id: 5`, status `in_progress` / `todo` atau keyword pencarian).
   - Jika task untuk pekerjaan tersebut belum ada, **otomatis buatkan task baru** (`create_task`) dengan status `in_progress`, cantumkan deskripsi jelas, dan pecah menjadi checklist `subtasks`.

3. **Penyelesaian & Logging Progres**:
   - Setelah pekerjaan selesai diimplementasikan, diverifikasi (build/test/lint lulus), dan siap di-commit/push, **otomatis perbarui progress dan status task**:
     - Gunakan `log_task_progress` untuk mencatat catatan penyelesaian.
     - Update subtask yang selesai (`completed_subtask_ids`) dan ubah status task utama menjadi `done` (`update_task_status`).

4. **Eksekusi Mandiri**:
   - Jalankan seluruh siklus task management ini (Cari -> Buat jika belum ada -> Kerjakan -> Verifikasi -> Mark Done) secara otonom di setiap giliran kerja tanpa perlu konfirmasi manual dari user untuk sekadar update task.

---

## 🏗️ Standar Arsitektur & Template Konvensi (@exa31/create-stack)

Repository ini disusun mengikuti arsitektur **Controller + Worker Agent** menggunakan template resmi `@exa31/create-stack`:

### 1. Controller Server (`backend/`)
- **Runtime & Framework**: Node.js 22+ / TypeScript (strict, CommonJS) + Express 5 REST API + Socket.io / WebSocket Server.
- **Database**: PostgreSQL (pg Pool) dengan schema migrations via `node-pg-migrate` di `backend/migrations/`.
- **Transactions 100% WAJIB untuk SEMUA Operasi Database**:
  - **Operasi Mutasi (`INSERT`, `UPDATE`, `DELETE`)**: WAJIB dieksekusi via `this.withTransaction` (dari `BaseRepository`) atau `db.withTransaction(async (client) => ...)`.
  - **Operasi Pembacaan (`SELECT`, query, get, list)**: WAJIB dieksekusi via `this.withReadOnlyTransaction` (dari `BaseRepository`) atau `db.withReadOnlyTransaction(async (client) => ...)`.
  - Dilarang keras manual `BEGIN` / `COMMIT` atau query tanpa helper transaksi.
- **Architecture Flow (Feature Modules)**:
  `HTTP Request → module routes.ts → validate() (req.validated) → service.ts → repository.ts → db pool`.
- **HTTP Response Envelope**:
  Seluruh respons API **WAJIB** menggunakan envelope standar `ApiResponse` (`src/types/response.ts`):
  `{ success: boolean, message: string, code: string, data: any, timestamp: string, requestId: string }`.
  Kode respon wajib didefinisikan secara semantik di `src/constants/codes.ts` (`RESPONSE_CODES`).
- **Validation**:
  Seluruh endpoint divalidasi ketat pada boundary route menggunakan Zod schema melalui middleware `validate({ body, query, params })`.
- **Code Style (ESLint)**:
  No semicolons, single quotes (`avoidEscape: true`), 2 spaces indent, `comma-dangle: never`, space before function parenthesis (`function name () {}`).

### 2. Worker Agent (`worker/`)
- **Runtime**: TypeScript / Node.js daemon.
- **Networking**: Koneksi outbound ke Controller via WebSocket (mendukung agent di balik NAT/home network tanpa public IP).
- **Process Group Isolation**: Setiap step dijalankan dalam OS Process Group (`setsid` / process group leader) agar pembatalan atau timeout dapat menterminasi seluruh process tree tanpa meninggalkan orphan/zombie.
- **Incremental Git Cache**: Cache repo lokal per project/service menggunakan `git fetch` + `git reset --hard <sha>`.
- **Dynamic Environment**: Dukungan file inject `$CI_ENV` dan output downstream `$CI_OUTPUT`.
- **Log Streaming & Masking**: Menangkap `stdout`/`stderr` baris demi baris, melakukan masking rahasia (`***`), dan streaming real-time ke Controller.
