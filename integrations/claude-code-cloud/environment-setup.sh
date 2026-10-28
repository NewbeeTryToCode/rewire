#!/bin/bash
# Setup script for a Claude Code cloud environment (claude.ai/code → environment → "Setup script").
# Installs the rewire binary once; the environment cache keeps it for later sessions.
#
# The cache is only built when the script finishes in about five minutes, so this builds without LTO
# (about 3 minutes on a cloud VM instead of 4.5). It fetches the source with git, which needs the rewire
# repository to be public, or REWIRE_GITHUB_TOKEN set in the environment's variables (a fine-grained,
# read-only token for that one repository).
set -euo pipefail

REPO="${REWIRE_REPO:-github.com/NewbeeTryToCode/rewire}"
REF="${REWIRE_REF:-main}"

if command -v rewire >/dev/null 2>&1; then
  echo "rewire already installed: $(rewire --version)"
  exit 0
fi

url="https://$REPO"
if [ -n "${REWIRE_GITHUB_TOKEN:-}" ]; then
  url="https://x-access-token:${REWIRE_GITHUB_TOKEN}@$REPO"
fi

export CARGO_PROFILE_RELEASE_LTO=false
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
export CARGO_PROFILE_RELEASE_OPT_LEVEL=1
export CARGO_NET_GIT_FETCH_WITH_CLI=true # use git (and its proxy settings) to fetch the source

cargo install --locked --git "$url" --branch "$REF" --root /usr/local rewire
rewire --version
