# Development Workflow

Efficient development practices for wayvid.

## Quick Checks

Local verification before pushing:

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all-features -- -D warnings

# Check
cargo check --all-features

# Test
cargo test --all-features
```

## Fast Iteration

### Use Debug Builds
```bash
cargo build  # Much faster than --release
./target/debug/wayvid
```

### Watch Mode
```bash
cargo install cargo-watch
cargo watch -x 'clippy --all-features'
```

### Faster Linker

**Install mold:**
```bash
# Arch
sudo pacman -S mold

# Ubuntu
sudo apt install mold
```

**Configure (~/.cargo/config.toml):**
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test '*'
```

### Specific Test
```bash
cargo test test_name
```

### With Output
```bash
cargo test -- --nocapture
```

## Logging

### Enable Debug Logs
```bash
RUST_LOG=debug wayvid
```

### Module-Specific
```bash
RUST_LOG=wayvid::video=trace wayvid
```

## Profiling

### CPU Profiling
```bash
cargo install flamegraph
cargo flamegraph --bin wayvid
```

### Memory Profiling
```bash
valgrind --tool=massif ./target/release/wayvid
```

## Documentation

### Build Docs
```bash
cargo doc --all-features --no-deps --open
```

### mdbook
```bash
cd docs
mdbook serve --open
```

## Git Workflow

### Branch Naming
- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(gui): Add video source browser
fix(mpv): Resolve memory leak
docs: Update installation guide
chore: Update dependencies
```

### Before Pushing
```bash
cargo fmt --all
cargo clippy --all-features -- -D warnings
cargo test --all-features
```

## IDE Setup

### VS Code
Recommended extensions:
- `rust-analyzer` - Language server
- `crates` - Dependency management
- `Error Lens` - Inline errors

**.vscode/settings.json:**
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

### Neovim
Use `rust-tools.nvim` with LSP:
```lua
require('rust-tools').setup({})
```

## Performance Tips

- Use `cargo build` for dev, `--release` for testing
- Enable incremental compilation (default)
- Use `cargo check` for faster feedback
- Run tests on relevant modules only
- Use `mold` linker for 2-3x faster linking

## CI/CD

GitHub Actions runs on every push:
- Format check
- Clippy (strict)
- Build (all features)
- Tests (all features)

Verify locally before pushing to save CI time.

## Troubleshooting

**Slow compilation:**
- Use `cargo check` instead of `build`
- Install `mold` linker
- Reduce feature set during development

**Test failures:**
- Check system dependencies
- Verify compositor is running
- Run with `RUST_LOG=debug`

**Memory issues:**
- Use `cargo clean` to clear cache
- Check for resource leaks with `valgrind`
