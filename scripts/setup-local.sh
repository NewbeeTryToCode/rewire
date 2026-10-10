#!/usr/bin/env bash
# Install rewire on this machine and connect it to Claude Code (and Codex if present).
# Usage: bash scripts/setup-local.sh [git-ref]
set -euo pipefail

REF="${1:-claude/stoic-heisenberg-urzdal}"
SRC="git+https://github.com/NewbeeTryToCode/rewire@${REF}"

say() { printf '\n==> %s\n' "$*"; }

say "Checking Python >= 3.11"
python3 -c 'import sys; assert sys.version_info >= (3, 11), sys.version' \
  || { echo "rewire needs Python 3.11+"; exit 1; }

say "Installing rewire from ${REF}"
if command -v uv >/dev/null 2>&1; then
  uv tool install --force "$SRC"
elif command -v pipx >/dev/null 2>&1; then
  pipx install --force "$SRC"
else
  echo "Install uv (https://docs.astral.sh/uv/) or pipx first."; exit 1
fi
command -v rewire >/dev/null || { echo "rewire is not on PATH (try: uv tool update-shell / pipx ensurepath)"; exit 1; }

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
  claude mcp add --scope user rewire -- rewire mcp 2>/dev/null || echo "(MCP server already registered)"
else
  echo "Claude Code not found; skipping. Consolidation also uses 'claude -p' by default:"
  echo "set provider = \"anthropic\" in $BRAIN/brain.toml and pip install 'rewire[anthropic]' instead."
fi

if command -v codex >/dev/null 2>&1; then
  say "Connecting Codex"
  rewire install codex
  codex mcp add rewire -- rewire mcp 2>/dev/null || echo "(MCP server already registered)"
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
