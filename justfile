# Pigeon — task runner

set dotenv-load

# Type-check the workspace
check:
    cargo check --workspace

# Debug build
build:
    cargo build --workspace

# Release build
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Lint with warnings as errors
clippy:
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Lint gate: clippy + format check
lint: clippy fmt-check

# Remove build artifacts
clean:
    cargo clean

# Full CI pipeline
ci: fmt-check clippy test

# --- Development ---

# Start local Postgres
dev-up:
    docker compose up -d

# Stop local Postgres
dev-down:
    docker compose down

# Stop local Postgres and remove data
dev-reset:
    docker compose down -v

# Run database migrations
dev-migrate:
    cargo run -p pigeon-server -- migrate

# Run the API server
dev-run:
    cargo run -p pigeon-server -- serve

# Watch mode: re-check on file changes (requires cargo-watch)
dev-watch:
    cargo watch -c -x 'check --workspace'
