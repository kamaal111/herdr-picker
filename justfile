alias fmt := format
alias fmt-c := format-check

# List available commands
@_default:
    just --list --unsorted

# Build the debug binary
build:
    cargo build --locked

# Build the release binary where the plugin manifest and PTY suite expect it
build-release:
    cargo build --locked --release --target-dir target

# Format Rust code
format:
    cargo fmt --all

# Check Rust formatting
format-check:
    cargo fmt --all -- --check

# Lint all Rust targets and features, treating warnings as errors
lint:
    cargo clippy --locked --all-targets --all-features -- -D warnings

# Run Rust tests without a Herdr server
test:
    cargo test --locked --all-features

# Smoke test the release binary in a PTY with a fake Herdr CLI (requires Python 3)
test-pty: build-release
    python3 tests/pty_smoke.py

# Run formatting and lint checks
quality: format-check lint

# Run all CI verification checks
ready: quality test test-pty

# Compare PNG baselines in the pinned Linux environment (requires Docker)
test-snapshots *args:
    sh tests/snapshots/run.sh {{args}}

# Verify real Herdr focus, remembered panes, popup teardown, and cancellation
test-focus:
    sh tests/snapshots/run.sh --focus-tests

# Intentionally update all PNG baselines, or specify case names (never in CI)
record-snapshots *cases:
    sh tests/snapshots/run.sh --record {{cases}}

# Build and register this checkout as a local Herdr plugin
plugin-link: build-release
    "${HERDR_BIN_PATH:-herdr}" plugin link "$PWD"

# Validate a live Herdr snapshot without changing focus
plugin-check: build-release
    ./target/release/herdr-picker check

# Rebuild and open the already linked picker in a running Herdr session
plugin-open: build-release
    "${HERDR_BIN_PATH:-herdr}" plugin action invoke herdr-picker.open

# Unregister the local plugin without deleting its files
plugin-unlink:
    "${HERDR_BIN_PATH:-herdr}" plugin unlink herdr-picker
