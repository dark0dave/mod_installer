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
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              git
              pkg-config
              pre-commit
              rust-analyzer
              rustc
              rustfmt
            ];
            buildInputs = with pkgs; [
              openssl
            ];
            env.RUSTC_VERSION = overrides.toolchain.channel;
 #           env.CARGO_HOME = "";
 #           env.RUSTUP_HOME = "";
          };
        });
      packages = forEachSystem (system: {
        default = pkgsFor.${system}.callPackage ./default.nix { };
      });
    };
}
