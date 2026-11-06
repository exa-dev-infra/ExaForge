# ExaForge: Lightweight Self-Hosted CI/CD Engine

<p align="center">
  <strong>Lightweight, high-performance, self-hosted CI/CD Engine built for high-velocity teams.</strong><br>
  100% Pure Rust &bull; Controller + Worker Agent &bull; Process Group Isolation &bull; Askama SSR &bull; KEDA Zero-Scaling
</p>

---

## 🚀 Key Features

- **Unified Single Binary / Dual-Mode CLI**:
  - `exaforge serve`: Starts the Controller (HTTP Server Axum 0.7, Web Dashboard SSR via Askama, REST API, and WebSocket Agent Hub).
  - `exaforge agent`: Starts a Worker Agent daemon connecting outbound to Controller via WebSocket (friendly to NAT and private networks without public IPs), advertising capability tags (`linux`, `docker`, `k8s`, `android`, `macos`).
- **Embedded Server-Side Rendering (Askama)**:
  - Zero JS build steps, zero npm, zero `node_modules` overhead.
  - Modern Dark Mode dashboard powered by Tailwind CSS + Alpine.js for live terminal streaming.
- **Granular Multi-Tier RBAC & Scoped Secrets**:
  - Hierarchical scoping: **Global Secrets** &rarr; **Project Secrets** &rarr; **Service Secrets** (`project:service`).
  - Roles: **Super Admin**, **Project Maintainer**, **Service Developer** (`coffee:fe` isolated from `coffee:be`), and **Viewer / QA** (read-only with staging deploy capability).
  - Secrets encrypted at-rest using **AES-256-GCM** (32-byte master key, 12-byte IV, 16-byte auth tag).
- **Git Webhook Receiver & Smart Pipeline Parser**:
  - HMAC SHA-256 webhook signature verification.
  - Multi-repo & Monorepo path filtering (e.g. `apps/fe/**`) and tag prefix routing (`fe-v*`).
  - `.ci.yaml` and `.ci.toml` pipeline configuration parser with **DAG Topological Sorting**, cycle detection, and dynamic variable interpolation (`${VAR_NAME}`).
- **Smart Queue & Concurrency Control**:
  - Scoped concurrency groups (`deploy-coffee-be`) with `cancel_in_progress = true` to automatically abort stale builds.
  - Offline agent queuing: pending jobs are held until an agent advertising matching capability tags connects.
- **Robust Worker Execution Engine**:
  - **Process Group Isolation**: Every step executes in its own OS Process Group (`setpgid`). Cancellations cleanly terminate the entire process tree (`-pgid` SIGTERM/SIGKILL) leaving zero zombie processes.
  - **Incremental Git Cache**: Fast checkouts using mirror repositories and hard reset (`git fetch` + `git reset --hard <sha>`).
  - **Dynamic Environment Injection**: `$CI_ENV` (intra-job) and `$CI_OUTPUT` (cross-job DAG dependency passing).
  - **Built-in Directives**: Native shell scripts (`run`) and container build (`container_build`) with registry auth and multi-tag pushing.
  - **Real-Time Secret Masking**: Live line-by-line `stdout`/`stderr` streaming with secret values masked as `***`.

---

## 🛠️ Quickstart

### 1. Environment Setup

Copy example environment variables:
```bash
cp .env.example .env
```
Ensure PostgreSQL is running (default `DATABASE_URL=postgres://postgres:postgres@localhost:54321/exaforge?sslmode=disable`).

### 2. Start Controller Server

```bash
# Start HTTP server, dashboard, and WebSocket hub on port 8080
cargo run -- serve
```
Open `http://localhost:8080` in your browser.  
Default Admin Credentials: `admin` / `adminsecret123`.

### 3. Connect a Worker Agent

Register the agent in the Controller dashboard or obtain an agent token (`agt_*`), then start the worker daemon:
```bash
cargo run -- agent \
  --controller-url ws://localhost:8080/api/v1/agents/ws \
  --token agt_your_agent_token_here \
  --name build-worker-01 \
  --tags linux,docker,k8s
```

---

## 📜 Sample Pipeline Configuration (`.ci.yaml`)

```yaml
name: "Production Pipeline"

concurrency:
  group: "deploy-coffee-be"
  cancel_in_progress: true

inputs:
  app_name:
    default: "coffee-backend"
    description: "Application name"
  rollout_timeout:
    default: "300"
    description: "Timeout for kubernetes rollout"

jobs:
  lint:
    name: "Linting"
    tags: ["linux"]
    steps:
      - name: "Run Clippy"
        run: "cargo clippy"

  test:
    name: "Unit & Integration Tests"
    tags: ["linux", "docker"]
    needs: ["lint"]
    steps:
      - name: "Test Execution"
        run: "cargo test --all"

  build:
    name: "Container Image Build"
    tags: ["linux", "docker"]
    needs: ["test"]
    steps:
      - name: "Docker Build"
        container_build:
          image: "registry.company.com/${app_name}"
          tags: ["latest", "${CI_COMMIT_SHA}"]

  deploy:
    name: "Kubernetes Deploy"
    tags: ["linux", "k8s"]
    needs: ["build"]
    steps:
      - name: "Kubectl Rollout"
        run: "kubectl rollout status deployment/${app_name} --timeout=${rollout_timeout}s"
```

---

## 🧪 Testing

Execute the comprehensive test suite (45 unit & integration tests):
```bash
# Run all tests
cargo test

# Run CI/CD engine integration tests specifically
cargo test --test cicd_engine_test
```

---

## 🐳 Kubernetes & Docker Deployment

ExaForge Controller is designed for continuous deployment to Kubernetes with KEDA HTTP zero-scaling (`scale to 0` when idle) and Cloudflare DNS proxy.

### Docker Build
```bash
docker build -t ghcr.io/exa-dev-infra/exaforge:latest .
```

### Kubernetes Centralized Config
See manifests and Terraform module in `templates/kubernetes-config/exaforge/`:
- `namespace.yaml`
- `bridges.yaml` (KEDA HTTP proxy bridge)
- `secret.yaml`
- `setup.sh`

---

## 📜 License

MIT License.
