
set dotenv-load := true

# Print available recipes
default:
    @just --list

[private]
cargo +args:
    cargo {{args}}

# Fetch dependencies
fetch:
    @just cargo fetch --locked

# Check source code format
check-format: fetch
    @just cargo fmt --all -- --check

# Enforce source code format
format: fetch
    @just cargo fmt --all

# Type-check source code
check: fetch
    @just cargo check --frozen --all-targets --all-features

# Check lints with Clippy
lint: check
    @just cargo clippy --frozen --all-targets --all-features -- -D warnings

# Build debug
build +args='--all-features': fetch
    @just cargo build --frozen {{args}}

# Run debug
run +args: (build "--all-features")
    @just cargo run --frozen --all-features {{args}}

# Build tests
build-tests +args='--all-features': fetch
    @just cargo test --frozen {{args}} --no-run

# Run tests
test +args='--all-features': (build-tests args)
    @just cargo test --frozen {{args}}

# Build release
build-release $RUST_WRAPPER="" +args='--all-features': fetch
    @just cargo auditable build --frozen {{args}} --release

# Clean
clean:
    @just cargo clean

# Clean release
clean-release:
    @just cargo clean --release

# Create DEB archive
deb +args='--all-features': (build-release args)
    @just cargo deb -v --no-build --no-strip --package house-dashboard

# Audit dependencies
audit:
    @just cargo audit --deny unsound --deny yanked

# Audit dependencies in binary
audit-binary binary:
    @just cargo audit bin --deny unsound --deny yanked '{{binary}}'
