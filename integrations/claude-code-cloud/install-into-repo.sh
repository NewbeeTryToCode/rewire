#!/usr/bin/env bash
# Add rewire's MCP server and hooks for Claude Code cloud sessions to a working repository.
# Usage: bash integrations/claude-code-cloud/install-into-repo.sh <path-to-working-repo>
#
# Writes (and merges into existing files, keeping what is there):
#   .claude/rewire/hook.sh   the hook and MCP wrapper (does nothing on your own machine except start the MCP server)
#   .claude/settings.json    hooks + permission rules that keep the agent from approving its own memories
#   .mcp.json                the `rewire` MCP server
# Then commit those three paths in the working repository.
set -euo pipefail

target="${1:?usage: install-into-repo.sh <path-to-working-repo>}"
here="$(cd "$(dirname "$0")" && pwd)"
[ -d "$target/.git" ] || { echo "$target is not a git repository"; exit 1; }

mkdir -p "$target/.claude/rewire"
install -m 755 "$here/hook.sh" "$target/.claude/rewire/hook.sh"

python3 - "$target" <<'PY'
import json, os, sys

target = sys.argv[1]
run = 'bash "$CLAUDE_PROJECT_DIR/.claude/rewire/hook.sh" '

def load(path):
    if not os.path.exists(path):
        return {}
    with open(path) as f:
        text = f.read().strip()
    return json.loads(text) if text else {}

def save(path, data):
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

# hooks: event -> (rewire event, timeout seconds)
events = {
    "SessionStart": ("session-start", 120),  # may clone the brain on a fresh VM
    "UserPromptSubmit": ("user-prompt", 10),
    "Stop": ("stop", 15),
    "PreCompact": ("pre-compact", 15),
    "SessionEnd": ("session-end", 15),
}
deny = []
for cmd in ["review approve", "remember --apply", "review reject", "forget", "undo", "remote set"]:
    deny.append(f"Bash(rewire {cmd}:*)")
for tool in ["Edit", "Write"]:
    deny.append(f"{tool}(//root/brain/**)")  # the brain on a cloud VM (home is /root)

path = os.path.join(target, ".claude", "settings.json")
settings = load(path)
hooks = settings.setdefault("hooks", {})
for event, (name, timeout) in events.items():
    entries = hooks.setdefault(event, [])
    command = run + name
    if not any(h.get("command") == command for e in entries for h in e.get("hooks", [])):
        entries.append({"hooks": [{"type": "command", "command": command, "timeout": timeout}]})
rules = settings.setdefault("permissions", {}).setdefault("deny", [])
rules.extend(r for r in deny if r not in rules)
save(path, settings)
print(f"wrote {path}")

path = os.path.join(target, ".mcp.json")
mcp = load(path)
mcp.setdefault("mcpServers", {})["rewire"] = {
    "command": "bash",
    "args": ["-c", 'exec bash "${CLAUDE_PROJECT_DIR:-.}/.claude/rewire/hook.sh" mcp'],
}
save(path, mcp)
print(f"wrote {path}")
PY

cat <<EOF

Done. Commit these in $target:
  git add .claude/rewire/hook.sh .claude/settings.json .mcp.json
  git commit -m "Use rewire memory in Claude Code cloud sessions"
EOF
