#!/usr/bin/env bash
# Setup script for Claude Code cloud sessions (claude.ai/code, the Claude app, `claude --cloud`): paste it into the
# cloud environment's settings (environment menu > Edit > Setup script) and set BRAIN below. Nothing else to set.
#
# BRAIN is your private brain repository. A new one, empty or with only GitHub's README and LICENSE, is fine: the
# first session turns it into a brain. Git goes through the cloud's GitHub proxy, so no token is needed once the
# Claude GitHub App can reach the repository.
set -euo pipefail

BRAIN="https://github.com/<you>/brain"
REF="zcode" # the rewire branch to install

cargo install --locked --force --git https://github.com/NewbeeTryToCode/rewire --branch "$REF" rewire
"$HOME/.cargo/bin/rewire" install cloud --remote "$BRAIN"
