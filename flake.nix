{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
      pkgsFor = nixpkgs.legacyPackages;
    in {
      devShells = forEachSystem (system:
        let
          pkgs = import nixpkgs { inherit system; };
          overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
        in {
          default =
            with pkgs;
            mkShell rec {
                nativeBuildInputs = [
                    cargo
                    clippy
                    git
                    pkg-config
                    pre-commit
                    rust-analyzer
                    rustc
                    rustfmt
                    libx11
                    libxcursor
                    libxrandr
                    libxi
                    libxcb
                    libxkbcommon
                    vulkan-loader
                    wayland
                ];
                buildInputs = [
                    openssl
                ];
                env.RUSTC_VERSION = overrides.toolchain.channel;
                env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
                shellHook = ''
                    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath nativeBuildInputs)}";
                '';
            };
        });
      packages = forEachSystem (system: {
        default = pkgsFor.${system}.callPackage ./default.nix { };
      });
    };
}
