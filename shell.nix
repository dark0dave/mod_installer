with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    cargo
    git
    gnupg
    pre-commit
    rustup
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
