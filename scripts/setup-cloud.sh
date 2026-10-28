#!/usr/bin/env bash
# Setup script for Claude Code cloud sessions (claude.ai/code, the Claude app, `claude --cloud`).
#
# Paste it into the cloud environment's settings (environment menu > Edit > Setup script) and set these
# environment variables there:
#
#   REWIRE_REMOTE=https://github.com/<you>/brain   # your private brain repository
#   REWIRE_REMOTE_TRUSTED=1                        # optional: its notes count as reviewed in the VM
#
# The script runs once and is cached as a filesystem snapshot: it installs rewire and connects Claude Code. The
# brain itself is fetched every session by the session-start hook (or by the MCP server, whichever starts first),
# through the cloud's GitHub proxy: no token is needed when the Claude GitHub App can reach the repository.
set -euo pipefail

REF="${REWIRE_REF:-main}"

say() { printf '\n==> %s\n' "$*"; }

command -v cargo >/dev/null 2>&1 || { echo "rewire builds with Rust: cargo is missing"; exit 1; }

say "Installing rewire from ${REF}"
cargo install --locked --force --git https://github.com/NewbeeTryToCode/rewire --branch "${REF}" rewire
REWIRE="$HOME/.cargo/bin/rewire"

# an older rewire on PATH (e.g. a Python shim from an earlier setup) would shadow this one in the agent's shell
for old in $(type -ap rewire); do
  if [ "$old" != "$REWIRE" ] && head -c 200 "$old" 2>/dev/null | grep -q python; then
    say "Removing an old Python rewire at $old"
    rm -f "$old" || true
  fi
done

say "Connecting Claude Code"
"$REWIRE" install claude-code --write
claude mcp remove --scope user rewire >/dev/null 2>&1 || true
claude mcp add --scope user rewire -- "$REWIRE" mcp

cat <<EOF

Done. Each new session fetches the brain from \$REWIRE_REMOTE into ~/brain, shows the "# rewire memory"
snapshot at start, and pushes what it learns back after consolidation.
EOF
