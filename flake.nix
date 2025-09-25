{
  description = "";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/2cd3cac16691a933e94276f0a810453f17775c28";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.mkShell {
            name = "rust-env";
            # Libs
            buildInputs = with pkgs; [
              openssl
              rustup
            ];
            # Tools
            nativeBuildInputs = with pkgs; [
              git
              pkg-config
              pre-commit
            ];
          };
        });
    };
}