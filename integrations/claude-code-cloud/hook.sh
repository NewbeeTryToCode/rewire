#!/usr/bin/env bash
# rewire hooks for Claude Code *cloud* sessions (claude.ai/code, the Claude app, `claude --cloud`).
#
# Committed to a working repository as .claude/rewire/hook.sh and called from that repository's
# .claude/settings.json. On your own machine it does nothing: there the user-level hooks written by
# `rewire install claude-code --write` already run, and running both would inject memory twice.
#
# Usage (from .claude/settings.json): bash "$CLAUDE_PROJECT_DIR/.claude/rewire/hook.sh" <event>
#   events: session-start | user-prompt | stop | pre-compact | session-end
#
# Settings, all optional, read from the cloud environment's variables:
#   REWIRE_BRAIN               brain directory                      (default: ~/brain)
#   REWIRE_DEVICE              device name in the event log         (default: cloud)
#   REWIRE_BRAIN_BRANCH        branch of THIS repository that holds the brain (default: rewire-brain)
#   REWIRE_REMOTE              a separate brain repository instead of the branch; it must be reachable from
#                              the session (cloud sessions only reach the repositories attached to them)
#   REWIRE_CLOUD_TRUST_BRAIN   1 = use the rules, preferences and skills from the brain without a review on
#                              this "device" (default 0: they wait for `rewire review`, see the guide)
#   REWIRE_CLOUD_CONSOLIDATE_EVERY  seconds between consolidations started from the Stop hook (default 1800)
set -u

event="${1:-}"

if [ "$event" = "mcp" ] && [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exec rewire mcp # on your own machine: the MCP server as usual (stdin is the MCP stream, not a hook payload)
fi
[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || { cat >/dev/null; exit 0; }
[ "$event" = "mcp" ] && payload="" || payload="$(cat)" # the hook's JSON, read once so it can be passed on

export REWIRE_BRAIN="${REWIRE_BRAIN:-$HOME/brain}"
export REWIRE_DEVICE="${REWIRE_DEVICE:-cloud}"
branch="${REWIRE_BRAIN_BRANCH:-rewire-brain}"
state="$HOME/.rewire-cloud"
mkdir -p "$state"
log="$state/hook.log"
note() { printf '%s %s %s\n' "$(date -u +%FT%TZ)" "$event" "$*" >>"$log"; }

# A notice for the agent at session start; other events stay silent.
say() { [ "$event" = "session-start" ] && printf '# rewire memory\n(%s)\n' "$*"; }

PATH="$HOME/.cargo/bin:/usr/local/bin:$PATH"
if ! command -v rewire >/dev/null 2>&1; then
  note "rewire binary missing"
  say "rewire is not installed in this cloud environment. Add the rewire setup script to the environment (see docs/cloud-sessions.md in the rewire repository)."
  exit 0
fi

ensure_brain() {
  # the MCP server and the SessionStart hook start together: one clones, the other waits for it
  exec 9>"$state/clone.lock"
  flock -w 120 9 || return 1
  ensure_brain_locked
  local rc=$?
  flock -u 9
  return $rc
}

ensure_brain_locked() {
  [ -f "$REWIRE_BRAIN/brain.toml" ] && return 0
  local url="${REWIRE_REMOTE:-}" tmp src
  local url="${REWIRE_REMOTE:-}" tmp
  if [ -z "$url" ]; then
    url="$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" remote get-url origin 2>/dev/null)" || return 1
    git ls-remote --exit-code --heads "$url" "$branch" >/dev/null 2>&1 || {
      note "no branch $branch on $url"
      return 2
    }
    # rewire clones a remote's default branch: make a local single-branch copy whose HEAD is the brain branch
    tmp="$state/brain-src"
    rm -rf "$tmp"
    git clone -q --single-branch -b "$branch" "$url" "$tmp" >>"$log" 2>&1 || return 1
    src="$tmp"
  else
    src="$url"
  fi
  if [ "${REWIRE_CLOUD_TRUST_BRAIN:-0}" = "1" ]; then
    # trusted: the brain as it is in git, rules and preferences active at once
    git clone -q "$src" "$REWIRE_BRAIN" >>"$log" 2>&1 || return 1
  else
    # the safe path: rewire's own clone checks the files and leaves rules, preferences and skills for a review
    rewire init --from "$src" --brain "$REWIRE_BRAIN" --device "$REWIRE_DEVICE" >>"$log" 2>&1 || return 1
  fi
  # pushes go back to the real remote, on the brain branch
  git -C "$REWIRE_BRAIN" remote set-url origin "$url"
  rm -rf "$state/brain-src"
  note "brain cloned from $url ($branch)"
}

case "$event" in
  mcp)
    ensure_brain || note "mcp: no brain"
    exec rewire mcp
    ;;
  session-start)
    ensure_brain
    case $? in
      0) ;;
      2) say "no brain yet: push one to the '$branch' branch of this repository (see docs/cloud-sessions.md)"; exit 0 ;;
      *) say "the brain could not be fetched (details in $log)"; exit 0 ;;
    esac
    [ -f "$state/last-consolidation" ] || date +%s >"$state/last-consolidation"
    printf '%s' "$payload" | rewire hook session-start 2>>"$log"
    ;;
  user-prompt)
    [ -f "$REWIRE_BRAIN/brain.toml" ] || exit 0
    printf '%s' "$payload" | rewire hook user-prompt 2>>"$log"
    ;;
  pre-compact | session-end)
    [ -f "$REWIRE_BRAIN/brain.toml" ] || exit 0
    printf '%s' "$payload" | rewire hook "$event" 2>>"$log"
    date +%s >"$state/last-consolidation"
    ;;
  stop)
    # A cloud VM is often reclaimed without a SessionEnd, taking the transcript with it. Queue the transcript
    # for consolidation (which pushes the brain when it is done) every so often while the session is alive.
    [ -f "$REWIRE_BRAIN/brain.toml" ] || exit 0
    every="${REWIRE_CLOUD_CONSOLIDATE_EVERY:-1800}"
    last="$(cat "$state/last-consolidation" 2>/dev/null || echo 0)"
    now="$(date +%s)"
    if [ $((now - last)) -ge "$every" ]; then
      echo "$now" >"$state/last-consolidation"
      printf '%s' "$payload" | rewire hook session-end 2>>"$log"
      note "consolidation queued"
    fi
    ;;
  *)
    note "unknown event"
    ;;
esac
exit 0
