{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.git
    pkgs.pre-commit
    pkgs.rustup
  ];
}
