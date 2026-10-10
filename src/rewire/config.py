import os
import socket
import tomllib
from dataclasses import dataclass, field

DEFAULT_TOML = """\
# rewire brain configuration
schema = 1
device = "{device}"

[activation]
model = "actr"
d = 0.7                  # decay; tuned in experiments/phase0
recall_beta = 0.3        # weight of activation in recall ranking (0 = off)

[budgets]                # characters (~4 chars per token)
constraints = 1500
preferences = 800
working = 2500

[consolidation]
# provider: "command" runs a CLI that reads the prompt on stdin and prints text,
#           e.g. "claude -p" (uses your existing Claude Code login)
# provider: "anthropic" uses the Anthropic SDK (pip install 'rewire[anthropic]')
provider = "command"
command = "claude -p --output-format text"
model = "claude-opus-5-5"
effort = "medium"
max_chunk_chars = 60000

# Map repositories to project slugs. Without a match the git top-level
# directory name is used.
# [projects.myproject]
# match = ["github.com/me/myproject", "~/code/myproject"]
"""


@dataclass
class Config:
    device: str = "default"
    d: float = 0.7
    recall_beta: float = 0.3
    budgets: dict = field(default_factory=lambda: {"constraints": 1500, "preferences": 800, "working": 2500})
    provider: str = "command"
    command: str = "claude -p --output-format text"
    model: str = "claude-opus-5-5"
    effort: str = "medium"
    max_chunk_chars: int = 60000
    projects: dict = field(default_factory=dict)


def default_device() -> str:
    return (socket.gethostname().split(".")[0] or "default").lower()


def render_default(device: str | None = None) -> str:
    return DEFAULT_TOML.format(device=device or default_device())


def load(path: str) -> Config:
    if not os.path.exists(path):
        return Config()
    with open(path, "rb") as fh:
        raw = tomllib.load(fh)
    act = raw.get("activation", {})
    con = raw.get("consolidation", {})
    cfg = Config()
    cfg.device = raw.get("device", cfg.device)
    cfg.d = float(act.get("d", cfg.d))
    cfg.recall_beta = float(act.get("recall_beta", cfg.recall_beta))
    cfg.budgets.update(raw.get("budgets", {}))
    cfg.provider = con.get("provider", cfg.provider)
    cfg.command = con.get("command", cfg.command)
    cfg.model = con.get("model", cfg.model)
    cfg.effort = con.get("effort", cfg.effort)
    cfg.max_chunk_chars = int(con.get("max_chunk_chars", cfg.max_chunk_chars))
    cfg.projects = raw.get("projects", {})
    return cfg
