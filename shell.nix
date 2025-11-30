{ pkgs ? import <nixpkgs> {} }:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.mkShell {
  name = manifest.name;
  # Libs
  buildInputs = with pkgs; [
    openssl
    rustup
  ];
  # Tools
  nativeBuildInputs = with pkgs; [
    clippy
    git
    pkg-config
    python312
    python312Packages.pyyaml
    pre-commit
    rust-analyzer
    rustfmt
  ];
  RUSTC_VERSION = overrides.toolchain.channel;
}
