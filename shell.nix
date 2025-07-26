with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    openssl
    pre-commit
    rustup
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];
}
