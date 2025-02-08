with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  nativeBuildInputs = [
    # Build-time Additional Dependencies
    cargo
    pkg-config
    rustc
  ];
  buildInputs = [
    # Run-time Additional Dependencies
    git
    gnupg
    pre-commit
    rustup
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
