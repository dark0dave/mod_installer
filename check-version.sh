#!/usr/bin/env bash
set -euo pipefail

function version_gt() { test "$(echo "$@" | tr " " "\n" | sort -V | head -n 1)" != "$1"; }

function main() {
  local tag_version=$(git describe --tags --abbrev=0);
  echo "Git tag version: ${tag_version}"
  local current_version=$(head -3 Cargo.toml | tail -1 |  sed -E "s/^version \= \"(.*)\"/\1/")
  echo "Cargo.toml version: v${current_version}"
  if [ "${tag_version}" == "v${current_version}" ]; then
    echo "Versions are the same"
    exit 0;
  fi

  if version_gt "v${current_version}" "${tag_version}"; then
    echo "Current version greater than tag version"
    exit 0;
  fi

  echo "Failed, tag version: ${tag_version} is greater than Cargo,toml: v${current_version}"
  exit 1;
}

main
