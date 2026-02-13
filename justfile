# Dev workflow commands for ryguessr

set dotenv-load

# List available recipes
default:
    @just --list

# ── Dev ─────────────────────────────────────────────────────────

# Run frontend dev server (Next.js)
frontend:
    cd web && npm run dev

# Run backend (Rust)
server:
    cd ryguessr && cargo run

# Run backend with auto-reload on file changes
server-watch:
    cd ryguessr && cargo watch -x run

# Run frontend and backend concurrently
dev:
    just frontend & just server-watch & wait

# ── Build ───────────────────────────────────────────────────────

# Build frontend for production
frontend-build:
    cd web && npm run build

# Build backend for production
server-build:
    cd ryguessr && cargo build --release

# ── Quality ─────────────────────────────────────────────────────

# Run all checks (test, clippy, fmt, lint)
check: test clippy fmt-check lint

# Run backend tests
test:
    cd ryguessr && cargo test

# Run clippy lints
clippy:
    cd ryguessr && cargo clippy --all-targets -- -D warnings

# Check Rust formatting
fmt-check:
   cd ryguessr && cargo fmt --check

# Format Rust code
fmt:
    cd ryguessr && cargo fmt

# Lint frontend
lint:
    cd web && npm run lint

# ── Setup ───────────────────────────────────────────────────────

# Install frontend dependencies
install:
    cd web && npm install
