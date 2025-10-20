{
  description = "Dynamic video wallpaper engine for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        wayvid = pkgs.rustPlatform.buildRustPackage rec {
          pname = "wayvid";
          version = "0.1.0";

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

          # Tests require Wayland compositor
          doCheck = false;

          meta = with pkgs.lib; {
            description = "Dynamic video wallpaper engine for Wayland";
            homepage = "https://github.com/yourusername/wayvid";
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

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            
            # Dependencies
            pkg-config
            wayland
            wayland-protocols
            mpv-unwrapped
            libGL
            libglvnd
            
            # Tools
            wayland-utils
            libva-utils
          ];

          shellHook = ''
            echo "wayvid development environment"
            echo "Run 'cargo build' to compile"
            echo "Run 'wayvid check' to verify system capabilities"
          '';
        };

        apps.default = flake-utils.lib.mkApp {
          drv = wayvid;
        };
      }
    );
}
