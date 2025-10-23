# wayvid AppImage Packaging

This directory contains the configuration and scripts for building wayvid as an AppImage - a universal Linux application format that runs on any distribution without installation.

## What is AppImage?

AppImage is a format for distributing portable software on Linux without requiring installation or root permissions. It's a single executable file that contains the application and all its dependencies.

**Advantages**:
- 🚀 **Universal**: Runs on any Linux distribution (Ubuntu, Fedora, Arch, Debian, etc.)
- 📦 **Self-contained**: Includes all dependencies (except system libraries)
- 🔒 **No root required**: Users can run it without sudo
- 🗑️ **Easy cleanup**: Delete the file to uninstall
- 🔄 **No conflicts**: Doesn't interfere with system packages

## Prerequisites

To build the AppImage, you need:

1. **Rust toolchain**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **appimagetool**: Download from [AppImageKit releases](https://github.com/AppImage/AppImageKit/releases)
   ```bash
   wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
     -O ~/.local/bin/appimagetool
   chmod +x ~/.local/bin/appimagetool
   ```

**Optional** (for optimization):
- **UPX**: For binary compression
  ```bash
  # Ubuntu/Debian
  sudo apt install upx-ucl
  
  # Arch Linux
  sudo pacman -S upx
  
  # Fedora
  sudo dnf install upx
  ```

- **ImageMagick**: For icon generation (if icon is missing)
  ```bash
  # Ubuntu/Debian
  sudo apt install imagemagick
  
  # Arch Linux
  sudo pacman -S imagemagick
  
  # Fedora
  sudo dnf install ImageMagick
  ```

## Building the AppImage

### Quick Build

```bash
cd packaging/appimage
chmod +x build-appimage.sh
./build-appimage.sh
```

This will:
1. Build wayvid in release mode
2. Create the AppDir structure
3. Copy binaries, libraries, and documentation
4. Generate the AppImage
5. Output: `build/wayvid-0.3.0-x86_64.AppImage`

### Build with Custom Version

```bash
./build-appimage.sh 0.3.1
```

### Build with Maximum Compression

```bash
# Install UPX first
sudo apt install upx-ucl  # or equivalent for your distro

# Build will automatically use UPX if available
./build-appimage.sh
```

## Testing the AppImage

### Automated Testing

```bash
chmod +x test-appimage.sh
./test-appimage.sh build/wayvid-0.3.0-x86_64.AppImage
```

The test script checks:
- ✅ Basic functionality (--version, --help)
- ✅ wayvid-ctl integration
- ✅ Dependency satisfaction
- ✅ File size
- ✅ Permissions
- ✅ Content inspection

### Manual Testing

```bash
# Run wayvid
./wayvid-0.3.0-x86_64.AppImage --version
./wayvid-0.3.0-x86_64.AppImage --help

# Run wayvid-ctl
./wayvid-0.3.0-x86_64.AppImage ctl status
./wayvid-0.3.0-x86_64.AppImage ctl play

# Actually start wayvid
./wayvid-0.3.0-x86_64.AppImage
```

### Testing on Different Distributions

It's recommended to test the AppImage on multiple distributions:

**Using Docker**:
```bash
# Ubuntu
docker run -it --rm -v $(pwd):/mnt ubuntu:22.04 /mnt/wayvid-0.3.0-x86_64.AppImage --version

# Fedora
docker run -it --rm -v $(pwd):/mnt fedora:39 /mnt/wayvid-0.3.0-x86_64.AppImage --version

# Arch Linux
docker run -it --rm -v $(pwd):/mnt archlinux:latest /mnt/wayvid-0.3.0-x86_64.AppImage --version

# Debian
docker run -it --rm -v $(pwd):/mnt debian:12 /mnt/wayvid-0.3.0-x86_64.AppImage --version
```

**Note**: These are basic tests. For full Wayland testing, you need a graphical environment.

## File Structure

```
packaging/appimage/
├── AppRun                 # Launcher script (handles wayvid and wayvid-ctl)
├── wayvid.desktop         # Desktop entry file
├── wayvid.png            # Application icon (256x256)
├── build-appimage.sh     # Build script
├── test-appimage.sh      # Test script
└── README.md             # This file

build/
└── wayvid.AppDir/        # AppDir structure (temporary)
    ├── AppRun            # Launcher
    ├── wayvid.desktop    # Desktop entry
    ├── wayvid.png        # Icon
    └── usr/
        ├── bin/
        │   ├── wayvid        # Main binary
        │   └── wayvid-ctl    # Control binary
        ├── lib/              # Bundled libraries (libmpv, etc.)
        ├── share/
        │   ├── applications/
        │   │   └── wayvid.desktop
        │   ├── icons/
        │   │   └── hicolor/256x256/apps/
        │   │       └── wayvid.png
        │   ├── wayvid/
        │   │   └── config.example.yaml
        │   └── doc/wayvid/
        │       ├── README.md
        │       ├── QUICKSTART.md
        │       └── ...
```

## AppRun Script

The `AppRun` script is the entry point for the AppImage. It:

1. **Detects the execution mode**:
   - Direct invocation: runs `wayvid`
   - `ctl` argument: runs `wayvid-ctl`
   - Symlink as `wayvid-ctl`: runs `wayvid-ctl`

2. **Sets up environment**:
   - `LD_LIBRARY_PATH`: Adds bundled libraries
   - `PATH`: Adds binary directory
   - `XDG_DATA_DIRS`: Adds data directory

**Usage examples**:
```bash
./wayvid.AppImage                  # Run wayvid
./wayvid.AppImage ctl status       # Run wayvid-ctl status
ln -s wayvid.AppImage wayvid-ctl   # Create symlink
./wayvid-ctl status                # Run via symlink
```

## Bundled Dependencies

The AppImage includes:

**Always bundled**:
- `wayvid` binary
- `wayvid-ctl` binary
- `libmpv.so` (video playback)
- `libwayland-client.so` (Wayland client)
- `libwayland-egl.so` (EGL integration)
- Configuration files and documentation

**Not bundled** (assumed to be on system):
- glibc (standard C library)
- Linux kernel libraries
- Graphics drivers (Mesa, NVIDIA, etc.)
- Hardware acceleration libraries (VA-API, etc.)

**Rationale**: System libraries like glibc and graphics drivers are distribution-specific and should use the host's versions for better compatibility.

## Size Optimization

The build script includes several optimizations:

1. **Cargo release profile** (`Cargo.toml`):
   ```toml
   [profile.release]
   opt-level = 3
   lto = "thin"
   codegen-units = 1
   strip = true
   ```

2. **UPX compression** (optional):
   - Compresses binaries with LZMA
   - Reduces size by ~50-70%
   - Slight startup time increase

3. **Selective bundling**:
   - Only includes necessary libraries
   - Excludes debug symbols

**Typical sizes**:
- Without UPX: ~30-40 MB
- With UPX: ~15-20 MB

## Publishing to GitHub Releases

### 1. Build for All Architectures

```bash
# x86_64
./build-appimage.sh 0.3.0

# aarch64 (if on ARM system or using cross-compilation)
# TODO: Add cross-compilation support
```

### 2. Generate Checksums

```bash
cd build
sha256sum wayvid-0.3.0-x86_64.AppImage > SHA256SUMS
sha256sum wayvid-0.3.0-aarch64.AppImage >> SHA256SUMS
```

### 3. Create GitHub Release

```bash
gh release create v0.3.0 \
  --title "wayvid v0.3.0" \
  --notes "See CHANGELOG.md for details" \
  build/wayvid-0.3.0-x86_64.AppImage \
  build/wayvid-0.3.0-aarch64.AppImage \
  build/SHA256SUMS
```

Or manually via GitHub web interface:
1. Go to https://github.com/YangYuS8/wayvid/releases
2. Click "Draft a new release"
3. Tag: `v0.3.0`
4. Upload AppImage files and checksums
5. Publish release

### 4. Update Documentation

Update `README.md` with download link:
```markdown
## Installation

### AppImage (Universal)

Download the latest AppImage from [Releases](https://github.com/YangYuS8/wayvid/releases):

\`\`\`bash
# Download
wget https://github.com/YangYuS8/wayvid/releases/download/v0.3.0/wayvid-0.3.0-x86_64.AppImage

# Make executable
chmod +x wayvid-0.3.0-x86_64.AppImage

# Run
./wayvid-0.3.0-x86_64.AppImage
\`\`\`
```

## Troubleshooting

### AppImage doesn't run

**Error**: `cannot execute binary file: Exec format error`
- **Cause**: Wrong architecture (e.g., trying to run x86_64 on ARM)
- **Solution**: Download the correct architecture (x86_64 or aarch64)

**Error**: `./wayvid.AppImage: Permission denied`
- **Cause**: File is not executable
- **Solution**: `chmod +x wayvid.AppImage`

### Missing dependencies

**Error**: `error while loading shared libraries: libmpv.so.2: cannot open shared object file`
- **Cause**: libmpv not bundled correctly
- **Solution**: Check `build-appimage.sh` and ensure `copy_lib "libmpv.so"` succeeds

### AppImage too large

- **Solution 1**: Install and use UPX compression
  ```bash
  sudo apt install upx-ucl
  ./build-appimage.sh
  ```
- **Solution 2**: Remove unnecessary bundled libraries
- **Solution 3**: Use `strip` on binaries (already done by default)

### Build fails on appimagetool

**Error**: `appimagetool: command not found`
- **Solution**: Install appimagetool:
  ```bash
  wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
    -O ~/.local/bin/appimagetool
  chmod +x ~/.local/bin/appimagetool
  export PATH="$HOME/.local/bin:$PATH"
  ```

### FUSE not available

**Error**: `Cannot mount AppImage, please check your FUSE setup`
- **Solution 1**: Install FUSE
  ```bash
  # Ubuntu/Debian
  sudo apt install fuse libfuse2
  
  # Arch Linux
  sudo pacman -S fuse2
  
  # Fedora
  sudo dnf install fuse
  ```
- **Solution 2**: Extract and run directly
  ```bash
  ./wayvid.AppImage --appimage-extract
  ./squashfs-root/AppRun
  ```

## CI/CD Integration

### GitHub Actions

Add to `.github/workflows/release.yml`:

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-appimage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y upx-ucl imagemagick
          wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
            -O /usr/local/bin/appimagetool
          chmod +x /usr/local/bin/appimagetool
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Build AppImage
        run: |
          cd packaging/appimage
          chmod +x build-appimage.sh
          ./build-appimage.sh ${GITHUB_REF#refs/tags/v}
      
      - name: Test AppImage
        run: |
          cd packaging/appimage
          chmod +x test-appimage.sh
          ./test-appimage.sh
      
      - name: Generate checksums
        run: |
          cd packaging/appimage/build
          sha256sum wayvid-*.AppImage > SHA256SUMS
      
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            packaging/appimage/build/wayvid-*.AppImage
            packaging/appimage/build/SHA256SUMS
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Maintenance

### Updating the AppImage

When releasing a new version:

1. **Update version** in `Cargo.toml`
2. **Build new AppImage**:
   ```bash
   ./build-appimage.sh <new-version>
   ```
3. **Test thoroughly**:
   ```bash
   ./test-appimage.sh build/wayvid-<new-version>-x86_64.AppImage
   ```
4. **Update checksums**:
   ```bash
   cd build
   sha256sum wayvid-<new-version>-*.AppImage > SHA256SUMS
   ```
5. **Create GitHub release**
6. **Update download links** in README.md

### Icon Updates

If you want to update the icon:

1. Create a 256x256 PNG icon
2. Save as `packaging/appimage/wayvid.png`
3. Rebuild AppImage

## References

- [AppImage Documentation](https://docs.appimage.org/)
- [AppImageKit GitHub](https://github.com/AppImage/AppImageKit)
- [Best Practices](https://github.com/AppImage/AppImageKit/wiki/AppImage-Best-Practices)
- [Bundling Guidelines](https://github.com/AppImage/pkg2appimage/blob/master/excludelist)

## License

Same as wayvid: MIT OR Apache-2.0
