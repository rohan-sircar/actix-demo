---
name: cargo-make
description: Cargo make commands for actix-demo project. Use when running lint, format, unit tests, integration tests, or flaky tests. Front-load: lint-check, format, test, it-test, flaky-test.
---

# Cargo Make Commands

All commands use `cargo make` (not raw `cargo`). Never run `cargo test` directly — it skips testcontainers shutdown hooks, leaving orphaned Docker containers.

## Commands

| Command | What it does |
|---------|-------------|
| `cargo make lint-check` | `cargo fmt --check` + `cargo clippy -- -D warnings` |
| `cargo make format` | `cargo fmt --all` |
| `cargo make test` | Unit tests only (`cargo test --lib`) |
| `cargo make it-test` | Integration tests (`cargo test --test integration`) |
| `cargo make flaky-test` | Flaky/integration tests with retries (WS and email tests) |

## Rules

- **Never truncate `cargo test` output** with `tail`, `head`, or pipes. Testcontainers cleanup runs after the process exits; truncating kills the pipe before shutdown hooks fire.
- If containers accumulate, run: `docker rm -f $(docker ps -aq)`
- Always run `cargo make lint-check` after making code changes.
- Use `cargo make test` for fast feedback during development.
- Use `cargo make it-test` for full integration coverage.
- Use `cargo make flaky-test` when email/WS tests fail intermittently.
