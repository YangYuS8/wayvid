# Development Scripts

Useful development and testing scripts.

## Available Scripts

### Development Checks
- `dev-check.sh` - Fast development checks (format, clippy)
- `pre-push-check.sh` - Pre-commit validation (tests, build)
- `quick-check.sh` - Minimal check (clippy only)

### Testing & Debugging
- `test-layout-modes.sh` - Test video scaling modes (Fill/Contain/Stretch/Centre)
- `test-swww-conflict.sh` - Test wallpaper manager conflict detection
- `test-workshop.sh` - Test Steam Workshop integration
- `test-workshop-mock.sh` - Mock Workshop data for testing

## Usage

### Development Checks
```bash
# Quick clippy check
./scripts/quick-check.sh

# Full development check (format + clippy)
./scripts/dev-check.sh

# Pre-push validation (all checks + tests + build)
./scripts/pre-push-check.sh
```

### Testing Layout Modes
```bash
# Test all layout modes interactively
./scripts/test-layout-modes.sh

# Test with specific video
./scripts/test-layout-modes.sh ~/Videos/wallpaper.mp4
```

This script helps verify video scaling behavior:
- **Fill**: Fills screen, crops edges (recommended for wallpapers)
- **Contain**: Shows full video, may have black bars
- **Stretch**: Fills screen, may distort video
- **Centre**: Original size, centered

### Testing Conflicts
```bash
# Test wallpaper manager conflict detection
./scripts/test-swww-conflict.sh
```

### Testing Workshop Integration
```bash
# Test with real Steam data
./scripts/test-workshop.sh

# Test with mock data (no Steam required)
./scripts/test-workshop-mock.sh
```

## Testing

The project uses Rust's built-in test framework. Run tests with:

```bash
cargo test
cargo test --all-features
```

Tests are located inline with the source code using `#[test]` attributes.
