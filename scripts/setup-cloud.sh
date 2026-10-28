#!/usr/bin/env bash
# Setup script for Claude Code cloud sessions (claude.ai/code, the Claude app, `claude --cloud`): paste it into the
# cloud environment's settings (environment menu > Edit > Setup script) and set BRAIN and AUTHOR below.
#
# BRAIN is your private brain repository. Add it to the environment's repositories too: a cloud session reaches only
# the private repositories attached to it. A new repository, empty or with only GitHub's README and LICENSE, is fine:
# the first session turns it into a brain. rewire itself comes from its public releases, so no login is needed.
#
# AUTHOR is who the brain's commits are by. A cloud VM's own git identity is the agent's, so without it your
# memories would be committed under that name. Use your GitHub no-reply address (Settings > Emails).
set -euo pipefail

BRAIN="https://github.com/<you>/brain"
AUTHOR="<you> <<id>+<you>@users.noreply.github.com>"

curl -fsSL https://raw.githubusercontent.com/NewbeeTryToCode/rewire/main/scripts/install.sh | sh
"$HOME/.local/bin/rewire" install cloud --remote "$BRAIN" --author "$AUTHOR"
