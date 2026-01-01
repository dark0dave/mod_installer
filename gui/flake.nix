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
        pkgs = pkgsFor.${system};
        dlopenLibraries = with pkgs; [
            libxkbcommon
            vulkan-loader
            wayland
          ];
          overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              git
              pkg-config
              pre-commit
              rustc
              rustfmt
            ];
            # additional libraries that your project
            # links to at build time, e.g. OpenSSL
            buildInputs = with pkgs; [
              openssl
            ];
            env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
            env.RUSTC_VERSION = overrides.toolchain.channel;
          };
        });
    };
}
