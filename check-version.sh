#!/usr/bin/env bash
set -euo pipefail

release_tag=$(git describe --tags --abbrev=0);
trap "rm -f Cargo.toml.test" SIGINT;
sed "s/^version \= .*/version = \"${release_tag/v/}\"/" Cargo.toml > Cargo.toml.test;
diff -sd Cargo.toml.test Cargo.toml;
if [ $? -eq 0 ]; then
  rm -f Cargo.toml.test;
  exit 0;
else
  exit 1;
fi
