#!/usr/bin/env bash
set -euo pipefail

function gitDirty() {
  # GIT_INDEX_FILE is defined only when running as a git pre-commit hook.
  # If run after commit, such as during CI, the variable is unset.
  if [ "${GIT_INDEX_FILE:-unset}" = 'unset' ]; then
    output="$(git status --porcelain)"
    readonly output

    if [ -n "${output}" ]; then
      echo "${output}"
      exit 1
    fi
  fi
}

function gitCheck() {
  local commit="$(git hash-object -t tree /dev/null)"
  readonly commit

  # Report errors based on git configuration.
  # Respect overrides in .gitattributes if present.
  git diff-index --check "${commit}"
}

function main() {
  gitDirty
  gitCheck
}

main
