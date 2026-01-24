{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/2cd3cac16691a933e94276f0a810453f17775c28";
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
                    xorg.libX11
                    xorg.libXcursor
                    xorg.libXrandr
                    xorg.libXi
                    xorg.libxcb
                    libxkbcommon
                    vulkan-loader
                    wayland
                ];
                buildInputs = [
                    openssl
                ];
                env.RUSTC_VERSION = overrides.toolchain.channel;
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
