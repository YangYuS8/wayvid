# Development Scripts

Useful development and testing scripts for wayvid v0.5 workspace.

## Available Scripts

### Development Checks
- `dev-check.sh` - Check development environment setup
- `quick-check.sh` - Fast workspace checks (format, clippy)
- `pre-push-check.sh` - Full pre-commit validation (tests, build)

## Usage

### Development Environment Check
```bash
# Check that all dependencies and tools are available
./scripts/dev-check.sh
```

### Quick Development Check
```bash
# Fast clippy and format check
./scripts/quick-check.sh
```

### Pre-push Validation
```bash
# Full validation before pushing (all checks + tests)
./scripts/pre-push-check.sh

# Skip tests for faster feedback
SKIP_TESTS=1 ./scripts/pre-push-check.sh
```

## Testing

The project uses Rust's built-in test framework across the workspace:

```bash
# Run all workspace tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p wayvid-core
cargo test -p wayvid-library
cargo test -p wayvid-gui
cargo test -p wayvid-ctl
cargo test -p wayvid-engine
```

## Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --workspace

# Run GUI
./target/release/wayvid-gui

# Run CLI
./target/release/wayvid-ctl status
```
