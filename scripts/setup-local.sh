#!/usr/bin/env bash
# Install rewire on this machine and connect it to Claude Code (and Codex if present).
# Usage: bash scripts/setup-local.sh [git-ref]
#   no argument: the latest release binary (no Rust toolchain needed); a git ref: build that branch or tag with cargo
set -euo pipefail

REF="${1:-}"

say() { printf '\n==> %s\n' "$*"; }

if [ -z "${REF}" ]; then
  say "Installing the latest rewire release"
  curl -fsSL https://raw.githubusercontent.com/NewbeeTryToCode/rewire/main/scripts/install.sh | sh
  export PATH="$HOME/.local/bin:$PATH"
else
  command -v cargo >/dev/null 2>&1 || { echo "building from a git ref needs Rust: install it from https://rustup.rs first"; exit 1; }
  say "Building rewire from ${REF}"
  cargo install --locked --force --git https://github.com/NewbeeTryToCode/rewire --branch "${REF}" rewire
  export PATH="$HOME/.cargo/bin:$PATH"
fi
command -v rewire >/dev/null || { echo "rewire is not on PATH"; exit 1; }
REWIRE="$(command -v rewire)"

BRAIN="${REWIRE_BRAIN:-$HOME/brain}"
if [ -f "$BRAIN/brain.toml" ]; then
  say "Brain already exists at $BRAIN"
else
  say "Creating brain at $BRAIN"
  rewire init
fi

if command -v claude >/dev/null 2>&1; then
  say "Connecting Claude Code (settings are backed up to ~/.claude/settings.json.bak-rewire)"
  rewire install claude-code --write
  claude mcp add --scope user rewire -- "$REWIRE" mcp 2>/dev/null || echo "(MCP server already registered)"
else
  echo "Claude Code not found; skipping. Consolidation also uses 'claude -p' by default:"
  echo "set provider = \"anthropic\" in $BRAIN/brain.toml and export ANTHROPIC_API_KEY instead."
fi

if command -v codex >/dev/null 2>&1; then
  say "Connecting Codex"
  rewire install codex
  codex mcp add rewire -- "$REWIRE" mcp 2>/dev/null || echo "(MCP server already registered)"
fi

say "Health check"
rewire doctor

cat <<EOF

Done. Start a new Claude Code session: the "# rewire memory" snapshot is injected
at start, and each session is consolidated in the background after it ends.

Useful commands:
  rewire review              # what is waiting for your approval
  rewire prime               # what the agent sees at session start
  tail $BRAIN/.state/consolidate.log
EOF
