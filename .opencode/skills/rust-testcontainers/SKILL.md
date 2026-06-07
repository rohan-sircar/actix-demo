---
name: rust-testcontainers
description: When running Rust integration tests that use testcontainers (Docker-based services like PostgreSQL, Redis, MinIO), never truncate test output with tail/head — containers will be orphaned. Avoid running the full test suite; only run tests for new or edited modules.
---

# Rust Testcontainers Test Running

When running Rust integration tests that use testcontainers (Docker-based services like PostgreSQL, Redis, MinIO), follow these rules to avoid orphaned containers and ensure reliable test execution.

## Core Rule: Never Truncate Test Output

**NEVER** use `tail`, `head`, or any command that truncates output when running `cargo test` with testcontainers. The cleanup phase happens after test output ends — if you truncate, the containers never finish tearing down.

```bash
# WRONG — orphaned containers will accumulate
cargo test --test integration 2>&1 | tail -20

# CORRECT — let the full output flow through
cargo test --test integration 2>&1
```

## When to Use `tail` / `head`

Only use output truncation for **non-test** commands:
- `cargo check`
- `cargo clippy` (without tests)
- `cargo fmt --check`
- `cargo build`

Testcontainers cleanup can take 10-60 seconds after the last test assertion passes. Truncating cuts this short.

## Typical Workflow for Rust Integration Tests

```bash
# 1. Compile first (fast, no containers)
cargo check --tests

# 2. Run tests module-by-module to reduce container launch pressure
#    (see guidance below)

# 3. Lint (no containers involved)
cargo make lint-check
```

## Running Tests Module-by-Module

**Avoid running the full test suite.** The test suite launches many Docker containers (PostgreSQL, Redis, MinIO, Mailpit) in parallel — running everything together can overwhelm the system and take significant time.

**Only run tests for new or edited modules.** When you make changes to a specific module, run only the tests covering that module. Do not run the full integration test suite unless explicitly asked.

```bash
# Run a specific test module
cargo test --test integration auth::oauth 2>&1
cargo test --test integration auth::session 2>&1
cargo test --test integration auth::registration 2>&1
cargo test --test integration actions::users 2>&1
cargo test --test integration actions::sessions 2>&1
cargo test --test integration misc 2>&1

# Run a specific test function
cargo test --test integration test_github_login_redirects 2>&1

# Run all tests in a sub-module
cargo test --test integration auth:: 2>&1
```

When asking the user to run tests, suggest the smallest relevant scope. Only run the full suite if explicitly requested.

## If Orphaned Containers Accumulate

Clean them up manually:

```bash
docker rm -f $(docker ps -aq)
```

## Why This Happens

testcontainers creates Docker containers per test session and removes them in a shutdown hook. The hook runs after `cargo test` exits, which happens after all output is written. `tail -N` exits after receiving N lines, killing the pipe and the child process before its cleanup code runs.
