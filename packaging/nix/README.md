# Nix Flake for wayvid

This directory contains the Nix flake configuration for building and developing wayvid.

## Features

- **Package**: Build wayvid as a Nix package
- **Development Shell**: Complete development environment with all dependencies
- **Apps**: Run wayvid and wayvid-ctl directly via flake
- **Cross-platform**: Works on any Linux system with Nix

## Quick Start

### Prerequisites

Install Nix with flakes enabled:

```bash
# Install Nix (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# Or use the official installer
sh <(curl -L https://nixos.org/nix/install) --daemon

# Enable flakes (if using standard Nix)
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### Install wayvid

#### Option 1: Direct run (no installation)

```bash
# Run wayvid directly from GitHub
nix run github:YangYuS8/wayvid

# Run with config
nix run github:YangYuS8/wayvid -- run --config ~/.config/wayvid/config.yaml

# Check system capabilities
nix run github:YangYuS8/wayvid -- check

# Run wayvid-ctl
nix run github:YangYuS8/wayvid#wayvid-ctl -- status
```

#### Option 2: Install to profile

```bash
# Install to user profile
nix profile install github:YangYuS8/wayvid

# Now wayvid is in PATH
wayvid check
wayvid run --config ~/.config/wayvid/config.yaml
```

#### Option 3: Add to NixOS configuration

Add to `/etc/nixos/configuration.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    wayvid.url = "github:YangYuS8/wayvid";
  };

  outputs = { self, nixpkgs, wayvid, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      modules = [
        {
          environment.systemPackages = [
            wayvid.packages.x86_64-linux.default
          ];
        }
      ];
    };
  };
}
```

#### Option 4: Add to Home Manager

Add to `home.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    wayvid.url = "github:YangYuS8/wayvid";
  };

  home.packages = [
    wayvid.packages.x86_64-linux.default
  ];

  # Optional: Enable systemd service
  systemd.user.services.wayvid = {
    Unit = {
      Description = "Wayland Video Wallpaper";
      After = [ "graphical-session.target" ];
    };
    Service = {
      ExecStart = "${wayvid.packages.x86_64-linux.default}/bin/wayvid run --config %h/.config/wayvid/config.yaml";
      Restart = "on-failure";
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
```

### Build Locally

```bash
# Clone repository
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid

# Build
nix build

# Result is in ./result
./result/bin/wayvid --version

# Install to profile
nix profile install .
```

## Development

### Enter Development Shell

```bash
# From repository directory
nix develop

# Or directly from GitHub
nix develop github:YangYuS8/wayvid
```

The development shell includes:
- Latest stable Rust toolchain
- rust-analyzer for IDE support
- cargo-watch for auto-rebuild
- All build and runtime dependencies
- Development tools (clippy, rustfmt)
- Hardware acceleration libraries

### Development Workflow

```bash
# Enter dev shell
nix develop

# Build
cargo build

# Run
cargo run --bin wayvid -- check

# Watch mode (auto-rebuild on changes)
cargo watch -x 'run --bin wayvid -- check'

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Check
cargo check
```

### Update Dependencies

```bash
# Update flake inputs
nix flake update

# Update Cargo dependencies
cargo update

# Rebuild
nix build
```

## Flake Structure

```
flake.nix
├── inputs
│   ├── nixpkgs (NixOS package collection)
│   ├── flake-utils (Multi-system helper)
│   └── rust-overlay (Latest Rust toolchains)
├── outputs
│   ├── packages
│   │   ├── default (wayvid package)
│   │   └── wayvid
│   ├── apps
│   │   ├── default (wayvid binary)
│   │   ├── wayvid
│   │   └── wayvid-ctl
│   └── devShells
│       └── default (development environment)
```

## Flake Commands

### Package Management

```bash
# Show flake metadata
nix flake show github:YangYuS8/wayvid

# Show package info
nix flake info github:YangYuS8/wayvid

# Build without installing
nix build github:YangYuS8/wayvid

# Install to profile
nix profile install github:YangYuS8/wayvid

# List installed packages
nix profile list

# Remove from profile
nix profile remove <index>
```

### Running Apps

```bash
# Run default app (wayvid)
nix run github:YangYuS8/wayvid

# Run specific app
nix run github:YangYuS8/wayvid#wayvid-ctl -- status

# Pass arguments
nix run github:YangYuS8/wayvid -- run --config /path/to/config.yaml
```

### Development

```bash
# Enter development shell
nix develop github:YangYuS8/wayvid

# Run command in dev shell without entering
nix develop github:YangYuS8/wayvid --command cargo build

# Update flake lock file
nix flake update github:YangYuS8/wayvid
```

## Customization

### Override Package Version

```nix
{
  wayvid = (import (builtins.fetchGit {
    url = "https://github.com/YangYuS8/wayvid";
    ref = "refs/tags/v0.3.0";
  })).packages.x86_64-linux.default;
}
```

### Add to Overlay

```nix
{
  nixpkgs.overlays = [
    (final: prev: {
      wayvid = (import (builtins.fetchGit {
        url = "https://github.com/YangYuS8/wayvid";
      })).packages.${prev.system}.default;
    })
  ];
}
```

## Troubleshooting

### Build Fails

**Error**: `cannot find -lmpv`
```bash
# Ensure mpv is in buildInputs
nix develop
echo $LD_LIBRARY_PATH
```

**Error**: Cargo.lock is outdated
```bash
# Update lock file
cargo update
nix build
```

### Runtime Issues

**Error**: `error while loading shared libraries: libmpv.so.2`
```bash
# Install mpv system-wide or use nix-shell
nix-shell -p mpv
./result/bin/wayvid check
```

**Error**: Wayland socket not found
```bash
# Ensure you're in a Wayland session
echo $WAYLAND_DISPLAY

# Should output: wayland-0 or similar
```

### Development Shell Issues

**Error**: `rust-analyzer` not found
```bash
# Exit and re-enter dev shell
exit
nix develop
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Nix Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
        with:
          extra_nix_config: |
            experimental-features = nix-command flakes
      - run: nix build
      - run: nix flake check
```

### Cachix Setup

```bash
# Install cachix
nix-env -iA cachix -f https://cachix.org/api/v1/install

# Use cache (if available)
cachix use wayvid

# Push to cache (maintainers only)
nix build
cachix push wayvid ./result
```

## Maintenance

### Updating the Flake

```bash
# Update nixpkgs and other inputs
nix flake update

# Test build
nix build

# Test dev shell
nix develop

# Commit updated flake.lock
git add flake.lock
git commit -m "chore: update flake inputs"
```

### Release Process

1. Update version in `flake.nix`
2. Update `Cargo.toml` and `Cargo.lock`
3. Build and test: `nix build`
4. Tag release: `git tag v0.3.0`
5. Push: `git push --tags`

## Support

- **Issues**: https://github.com/YangYuS8/wayvid/issues
- **Nix Discourse**: https://discourse.nixos.org/
- **Matrix**: #nix:matrix.org

## License

Flake configuration is public domain. wayvid itself is dual-licensed under MIT or Apache-2.0.
