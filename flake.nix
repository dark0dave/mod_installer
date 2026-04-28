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
          remote = builtins.fetchTarball {
            url = "https://github.com/jdx/hk/archive/refs/tags/v1.43.0.tar.gz";
            sha256 = "0m7xjcsc7rv8pr3pyq5dx1j00bl51ik30ci51i4s11n0b7fqiix8";
          };
          hk = pkgs.callPackage (remote + "/default.nix") { };
        in {
          default =
            with pkgs;
            mkShell rec {
                nativeBuildInputs = [
                    cargo
                    clippy
                    hk
                    codespell
                    git
                    pkg-config
                    rust-analyzer
                    rustc
                    rustfmt
                    yamlfmt
                ];
                buildInputs = [
                    openssl
                ];
                env.RUSTC_VERSION = overrides.toolchain.channel;
                env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };
        });
      packages = forEachSystem (system: {
        default = pkgsFor.${system}.callPackage ./default.nix { };
      });
    };
}
