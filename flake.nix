{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    hk = {
      url = "github:jdx/hk/v1.44.2";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      hk,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
        in
        {
          default =
            with pkgs;
            mkShell rec {
              nativeBuildInputs = [
                cargo
                clippy
                hk.packages.${system}.default
                nixfmt
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
              env.HK_PKL_BACKEND = "pklr";
              env.OCAMLRUNPARAM = "s=16M,o=500,O=1000000";
              env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              env.RUSTC_VERSION = overrides.toolchain.channel;
            };
        }
      );
      packages = forEachSystem (system: {
        default = pkgsFor.${system}.callPackage ./default.nix { };
      });
      formatter = forEachSystem (system: nixpkgs.${system}.nixfmt);
    };
}
