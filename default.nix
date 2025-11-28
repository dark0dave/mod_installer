{ pkgs ? import <nixpkgs> { } }:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  buildInputs = with pkgs; [
    openssl
    cargo
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  RUSTC_VERSION = overrides.toolchain.channel;
}
