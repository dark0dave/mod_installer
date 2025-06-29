{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "mod_installer";
  version = "10.2.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  buildInputs = with pkgs; [
    openssl
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
}
