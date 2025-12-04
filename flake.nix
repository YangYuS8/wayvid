{
  description = "Dynamic video wallpaper engine for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.minimal;
          rustc = pkgs.rust-bin.stable.latest.minimal;
        };
        
        wayvid = rustPlatform.buildRustPackage rec {
          pname = "wayvid";
          version = "0.4.5-alpha.2";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
          ];

          buildInputs = with pkgs; [
            wayland
            mpv-unwrapped
            libGL
            libglvnd
          ];

          # Tests are currently minimal
          doCheck = false;

          postInstall = ''
            # Install systemd service
            install -Dm644 systemd/wayvid.service \
              $out/lib/systemd/user/wayvid.service

            # Install example config
            install -Dm644 configs/config.example.yaml \
              $out/share/wayvid/config.example.yaml

            # Install documentation
            install -Dm644 README.md $out/share/doc/wayvid/README.md
            install -Dm644 CHANGELOG.md $out/share/doc/wayvid/CHANGELOG.md
          '';

          meta = with pkgs.lib; {
            description = "Dynamic video wallpaper engine for Wayland compositors";
            homepage = "https://github.com/YangYuS8/wayvid";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
            platforms = platforms.linux;
            mainProgram = "wayvid";
          };
        };
      in
      {
        packages = {
          default = wayvid;
          wayvid = wayvid;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = wayvid;
          };
          wayvid = flake-utils.lib.mkApp {
            drv = wayvid;
          };
          wayvid-ctl = flake-utils.lib.mkApp {
            drv = wayvid;
            name = "wayvid-ctl";
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ wayvid ];
          
          packages = with pkgs; [
            # Rust toolchain
            rust-bin.stable.latest.default
            rust-analyzer
            
            # Development tools
            cargo-watch
            cargo-edit
            clippy
            rustfmt
            
            # Build dependencies
            pkg-config
            
            # Runtime dependencies for testing
            wayland
            wayland-protocols
            mpv-unwrapped
            libGL
            libglvnd
            
            # Optional hardware acceleration
            libva
            intel-media-driver
            
            # Tools
            wayland-utils
            libva-utils
          ];

          shellHook = ''
            echo "🚀 wayvid development environment"
            echo ""
            echo "Available commands:"
            echo "  cargo build           - Build the project"
            echo "  cargo run --bin wayvid -- check  - Check system capabilities"
            echo "  cargo run --bin wayvid -- run    - Run wayvid"
            echo "  cargo run --bin wayvid-ctl       - Control wayvid"
            echo "  cargo test            - Run tests"
            echo "  cargo clippy          - Run linter"
            echo "  cargo fmt             - Format code"
            echo "  cargo watch -x run    - Auto-rebuild on changes"
            echo ""
            echo "Environment:"
            echo "  Rust:    $(rustc --version)"
            echo "  Cargo:   $(cargo --version)"
            echo ""
          '';

          # Set environment variables for development
          RUST_BACKTRACE = "1";
          RUST_LOG = "wayvid=debug";
        };
      }
    );
}
