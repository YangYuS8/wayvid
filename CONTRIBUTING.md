# Contributing to wayvid

Thank you for your interest in contributing to wayvid! This document provides guidelines and information for contributors.

## Development Status

wayvid is currently in **M1 MVP** stage. The core architecture is established, but many features are still in development. See the [Roadmap](README.md#roadmap) for planned work.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Wayland development libraries
- libmpv development libraries
- OpenGL/EGL development libraries

See [README.md](README.md#from-source-recommended-for-now) for detailed installation instructions.

### Building

```bash
# Clone and build
git clone https://github.com/yourusername/wayvid.git
cd wayvid
cargo build

# Run tests
cargo test

# Run locally
cargo run -- check
cargo run -- run --config configs/config.example.yaml
```

### Development Environment

Use the provided check script:

```bash
./scripts/dev-check.sh
```

## Code Structure

```
src/
├── main.rs           # Entry point, CLI
├── config.rs         # Configuration management
├── core/             # Core logic
│   ├── layout.rs     # Layout calculation
│   └── types.rs      # Shared types
├── backend/          # Platform backends
│   └── wayland/      # Wayland implementation
├── video/            # Video playback
│   └── mpv.rs        # libmpv integration
└── ctl/              # Control/utilities
    └── check.rs      # Capability checking
```

## Coding Guidelines

### Rust Style

- Follow standard Rust style guide
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add documentation comments for public APIs

```rust
/// Calculate layout transform for video rendering
///
/// # Arguments
/// * `mode` - The layout mode (Fill, Contain, etc.)
/// * `video_width` - Source video width in pixels
/// * `video_height` - Source video height in pixels
/// * `output_width` - Target output width in pixels
/// * `output_height` - Target output height in pixels
///
/// # Returns
/// Layout transform with source and destination rectangles
pub fn calculate_layout(
    mode: LayoutMode,
    video_width: i32,
    video_height: i32,
    output_width: i32,
    output_height: i32,
) -> LayoutTransform {
    // ...
}
```

### Error Handling

- Use `anyhow::Result` for function returns
- Use `thiserror` for custom error types
- Provide context with `.context()` or `anyhow!()` macro
- Log errors appropriately with `tracing`

```rust
use anyhow::{Context, Result};

pub fn load_video(path: &Path) -> Result<Video> {
    let file = std::fs::read(path)
        .with_context(|| format!("Failed to read video file: {:?}", path))?;
    
    parse_video(&file)
        .context("Failed to parse video format")
}
```

### Logging

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error};

info!("Starting video playback on output {}", output_name);
debug!("Video dimensions: {}x{}", width, height);
warn!("Hardware decode not available, falling back to software");
error!("Failed to create EGL context: {}", e);
```

## Testing

### Unit Tests

Add unit tests for core logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_layout_wider_video() {
        let transform = calculate_layout(
            LayoutMode::Fill,
            1920, 1080, // 16:9 video
            1024, 768,  // 4:3 output
        );
        assert_eq!(transform.dst_rect, (0, 0, 1024, 768));
    }
}
```

### Integration Tests

Wayland tests require a running compositor. For CI, we skip these:

```rust
#[test]
#[ignore] // Requires Wayland compositor
fn test_layer_surface_creation() {
    // ...
}
```

Run with:
```bash
cargo test -- --ignored
```

## Pull Request Process

1. **Fork and create a branch**
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/my-fix
   ```

2. **Make your changes**
   - Write code following style guidelines
   - Add tests where applicable
   - Update documentation if needed

3. **Test your changes**
   ```bash
   cargo fmt --all
   cargo clippy --all-features
   cargo test --all-features
   cargo build --release
   ```

4. **Commit with clear messages**
   ```
   Add OpenGL rendering for video frames
   
   - Implement mpv_render_context integration
   - Add FBO rendering to layer surface
   - Support layout transformations
   
   Closes #42
   ```

5. **Push and create PR**
   - Provide clear description of changes
   - Reference related issues
   - Include testing steps if applicable

## Areas Needing Help

### High Priority (M2)
- **OpenGL/EGL Integration**: Full mpv_render_context setup
- **Frame Callbacks**: Proper Wayland frame timing
- **Multi-Output**: Hotplug detection and management
- **Testing**: More comprehensive test coverage

### Medium Priority (M3)
- **WE Importer**: Parse Wallpaper Engine project files
- **Packaging**: Flatpak, deb, rpm scripts
- **Documentation**: More examples and troubleshooting

### Nice to Have (M4)
- **Performance**: Profiling and optimization
- **IPC**: Control interface (D-Bus or Unix socket)
- **Static Images**: Integration with image wallpaper tools
- **Color Management**: ICC profile support

## Communication

- **Issues**: For bug reports and feature requests
- **Discussions**: For questions and ideas
- **Pull Requests**: For code contributions

## Code Review

All submissions require review. We'll check for:

- Code quality and style
- Test coverage
- Documentation
- Performance implications
- Security considerations

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

## Questions?

Feel free to open an issue or discussion if you have questions about contributing!

---

Thank you for helping make wayvid better! 🚀
