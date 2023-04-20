#!/usr/bin/env bash
set -euxo pipefail

NAME="${1}"
shift

for arg in "$@"; do
  if [ "${next_target:-}" = 1 ]; then
    next_target=
    TARGET="$arg"
    continue
  fi
  case "$arg" in
  --target)
    next_target=1
    ;;
  *) ;;
  esac
done

RUST_TRIPLE=${TARGET:-$(rustc -vV | grep ^host: | cut -d ' ' -f2)}

if [ "${CROSS:-}" = "1" ]; then
  cross build "$@"
else
  cargo build "$@"
fi

mkdir -p "dist/bin"
cp "target/${RUST_TRIPLE}/release/${NAME}" "dist/bin/${NAME}"
