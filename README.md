# actix-demo

A Rust JSON API built with Actix-Web, featuring user authentication, WebSocket support, background job execution, and object storage integration.

## Features

- **User Management** - Registration, login, profile updates, account deletion (soft delete), and user search with pagination
- **Authentication** - JWT-based auth via HTTP-only cookies, multi-session management with login/logout per device
- **WebSocket** - Real-time communication over authenticated WebSocket connections with heartbeat and session refresh
- **Background Jobs** - Run external binaries as background jobs with real-time output streaming via Redis PubSub, abort support
- **File Storage** - User avatar upload/download/delete backed by MinIO (S3-compatible object storage)
- **Rate Limiting** - Configurable Redis-backed rate limiting per endpoint category (auth, API, public)
- **Health Checks** - Multi-service health monitoring (PostgreSQL, Redis, Loki, Prometheus) with dependency status reporting
- **Observability** - Prometheus metrics, structured JSON/logging with tracing-loki integration to Grafana Loki
- **Session Management** - Configurable session expiration, renewal policies, concurrent session limits, and automatic cleanup worker

## Tech Stack

- **Framework**: Actix-Web 4
- **Database**: PostgreSQL (via Diesel ORM with r2d2 connection pooling)
- **Cache/Sessions**: Redis (with connection manager and pubsub)
- **Object Storage**: MinIO (S3-compatible)
- **Auth**: JWT (jwt-simple), bcrypt password hashing
- **Real-time**: WebSocket (actix-ws), Redis PubSub
- **Background Jobs**: process-stream with Redis-based abort channels
- **Monitoring**: Prometheus metrics, Grafana Loki logging, Grafana dashboards
- **Migrations**: Diesel migrations

## Project Structure

```
src/
  actions/        # Database operations (CRUD queries)
  models/         # Data types and domain models
  routes/         # HTTP/WebSocket endpoint handlers
  utils/          # Shared utilities (auth, caching, image validation)
  workers/        # Background workers (session cleanup)
  config.rs       # Environment-based configuration
  errors.rs       # Domain error types
  health.rs       # Health check implementations
  metrics.rs      # Prometheus metrics definitions
  telemetry.rs    # Tracing/span configuration
  lib.rs          # App setup and route registration
  main.rs         # Application entrypoint
migrations/       # Diesel database migrations
static/           # Static files
curls/            # Example curl/httpie commands
tests/            # Integration tests
```

## Getting Started

### Prerequisites

- Rust toolchain (edition 2021)
- Docker & Docker Compose (for development dependencies)

### Development Dependencies

The project runs alongside several services via Docker Compose:

| Service     | Port  | Purpose                    |
|-------------|-------|----------------------------|
| PostgreSQL  | 5432  | Primary database           |
| Redis       | 6379  | Sessions, caching, PubSub  |
| MinIO       | 9000  | Object storage (API)       |
| MinIO Console | 9001 | Object storage (UI)        |
| Prometheus  | 9090  | Metrics scraping           |
| Loki        | 3100  | Log aggregation            |
| Grafana     | 3000  | Dashboards & visualization |

### Quick Start (development)

1. Start all dependencies (comment out actix-demo app entry):
   ```bash
   docker compose up -d
   ```
2. Set postgres URL in .env file
3. Run `diesel migrations run` (install diesel-cli)
4. Start the application:
   ```bash
   cargo run
   ```

The API will be available at `http://localhost:7800`.

### Development Tasks

Tasks are defined in `Makefile.toml` (cargo-make):

| Task           | Description                        |
|----------------|------------------------------------|
| `cargo make watch`    | Run with hot reload              |
| `cargo make format`   | Format code with rustfmt         |
| `cargo make lint-check` | Run format check + clippy      |
| `cargo make compile`  | Build the project              |
| `cargo make test`     | Run unit tests (lib)         |
| `cargo make it-test`  | Run integration tests      |
| `cargo make stage`    | Full pipeline: lint + build + test |

Alternatively, use `bacon` for automatic background testing.

## API Endpoints

### Public

| Method | Path                              | Description                    |
|--------|-----------------------------------|--------------------------------|
| POST   | `/api/registration`               | Register a new user            |
| POST   | `/api/login`                      | Login (sets auth cookie)       |
| POST   | `/api/logout`                     | Logout (clears current session)|
| GET    | `/api/public/users`               | List all users (paginated)     |
| GET    | `/api/public/users/search`        | Search users                   |
| GET    | `/api/public/users/{user_id}`     | Get user by ID                 |
| GET    | `/api/public/avatars/{user_id}`   | Get user avatar                |
| GET    | `/api/public/metrics/cmd`         | Job metrics                    |
| GET    | `/api/public/build-info`          | Build information              |
| GET    | `/ws`                             | WebSocket connection           |
| GET    | `/hc`                             | Health check                   |

### Authenticated (requires `X-AUTH-TOKEN` cookie)

| Method | Path                              | Description                        |
|--------|-----------------------------------|------------------------------------|
| GET    | `/api/users`                      | Get my profile                     |
| PATCH  | `/api/users`                      | Update my profile                  |
| POST   | `/api/users/me/delete`            | Delete my account (soft delete)    |
| PUT    | `/api/avatars`                    | Upload avatar                      |
| DELETE | `/api/avatars`                    | Delete avatar                      |
| GET    | `/api/sessions`                   | List active sessions               |
| DELETE | `/api/sessions/{session_id}`      | Revoke a specific session          |
| POST   | `/api/sessions/revoke-others`     | Revoke all other sessions          |
| POST   | `/api/cmd`                        | Run a background command/job       |
| GET    | `/api/cmd/{job_id}`               | Get job status                     |
| DELETE | `/api/cmd/{job_id}`               | Abort a running job                |

## Configuration

All configuration is via environment variables prefixed with `ACTIX_DEMO_`. Key variables:

| Variable                                    | Default         | Description                          |
|---------------------------------------------|-----------------|--------------------------------------|
| `DATABASE_URL`                              | -               | PostgreSQL connection string         |
| `REDIS_URL`                                 | -               | Redis connection string              |
| `JWT_KEY`                                   | -               | Secret key for JWT signing           |
| `MINIO_ENDPOINT`                            | -               | MinIO/S3 endpoint URL                |
| `MINIO_BUCKET_NAME`                         | -               | Bucket for storing avatars           |
| `LOKI_URL`                                  | -               | Grafana Loki URL for log shipping    |
| `PROMETHEUS_URL`                            | -               | Prometheus URL                       |
| `LOGGER_FORMAT`                             | pretty          | Log format: `pretty` or `json`       |
| `RATE_LIMIT_AUTH_MAX_REQUESTS`              | 5               | Max login attempts per window        |
| `RATE_LIMIT_API_MAX_REQUESTS`               | 500             | Max authenticated API requests       |
| `RATE_LIMIT_API_PUBLIC_MAX_REQUESTS`        | 15              | Max public API requests              |
| `SESSION_EXPIRATION_SECS`                   | 86400           | Session TTL in seconds               |
| `MAX_CONCURRENT_SESSIONS`                   | 5               | Max sessions per user                |
| `JOB_BIN_PATH`                              | /bin/echo       | Path to allowed command binary       |
| `HASH_COST`                                 | 8               | Bcrypt work factor                   |
| `TIMEZONE`                                  | UTC             | Default timezone                     |

See `.env` for the full list of configuration options.

## Testing

```bash
# Unit tests
cargo test --lib

# Integration tests (requires Docker services running)
cargo test --test integration

# Run flaky/slow tests separately
cargo test --test integration -- --ignored
```

Integration tests use `testcontainers` to spin up PostgreSQL, Redis, and MinIO containers.

## License

AGPLv3
