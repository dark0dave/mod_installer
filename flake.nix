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
            url = "https://github.com/jdx/hk/archive/refs/tags/v1.44.2.tar.gz";
            sha256 = "0a045ixfkj79f9nkiw598vraifv1dj744c6wjxbfaz478xli37rw";
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
                env.HK_PKL_BACKEND = "pklr";
            };
        });
      packages = forEachSystem (system: {
        default = pkgsFor.${system}.callPackage ./default.nix { };
      });
    };
}
