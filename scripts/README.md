# Development Scripts

Useful development and testing scripts.

## Available Scripts

- `dev-check.sh` - Fast development checks (format, clippy)
- `pre-push-check.sh` - Pre-commit validation (tests, build)
- `quick-check.sh` - Minimal check (clippy only)

## Usage

```bash
# Quick clippy check
./scripts/quick-check.sh

# Full development check (format + clippy)
./scripts/dev-check.sh

# Pre-push validation (all checks + tests + build)
./scripts/pre-push-check.sh
```

## Testing

The project uses Rust's built-in test framework. Run tests with:

```bash
cargo test
cargo test --all-features
```

Tests are located inline with the source code using `#[test]` attributes.
