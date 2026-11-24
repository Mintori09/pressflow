# Default recipe: list available commands
default:
    @just --list

# Run the application
run *args:
    cargo run -- {{args}}

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Build with the release-fast profile
build-fast:
    cargo build --profile release-fast

# Check compilation across all targets
check:
    cargo check --all-targets

# Format all code
fmt:
    cargo fmt --all

# Check code formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run Clippy linter with warnings treated as errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test *args:
    cargo test --all-targets {{args}}

# Run all CI checks locally (format, lint, test)
ci: fmt-check lint test

# Run security and license audits
audit:
    cargo audit
    cargo deny check

# Clean build artifacts
clean:
    cargo clean
