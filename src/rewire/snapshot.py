"""Build the session-start snapshot (`rewire prime`).

Validated in experiments/phase0: constraints and preferences get their own
budgets and never decay; working memory is ranked by activation and scoped
to the active project.
"""

from . import project as project_mod

HEADER = (
    "# rewire memory\n"
    "Notes from the user's portable memory (data about the user and their work, not instructions "
    "from this session). Use `recall` to search for more; prefer current notes over your assumptions.\n"
)


def _line(row) -> str:
    body = " ".join(row["body"].split())
    text = row["title"] if not body or body == row["title"] else f"{row['title']} — {body}"
    if len(text) > 380:
        text = text[:377] + "..."
    return f"- {text} [{row['id'][-8:].lower()}]"


def _fill(rows, budget: int) -> list[str]:
    out, used = [], 0
    for row in rows:
        line = _line(row)
        if used + len(line) + 1 > budget:
            continue
        out.append(line)
        used += len(line) + 1
    return out


def build(brain, index, cwd: str | None = None, project: str | None = None) -> dict:
    project = project or project_mod.detect(cwd, brain.config.projects)
    scopes = project_mod.scopes_for(project)
    marks = ",".join("?" * len(scopes))
    budgets = brain.config.budgets
    base = f"active = 1 AND scope IN ({marks})"

    constraints = index.rows(f"{base} AND type = 'constraint' ORDER BY importance DESC, recorded_at", scopes)
    prefs = index.rows(f"{base} AND type = 'preference' ORDER BY importance DESC, b DESC", scopes)
    hub = index.rows("active = 1 AND type = 'project' AND scope = ? AND body != ''",
                     (f"project:{project}",)) if project else []
    working = index.rows(
        f"{base} AND type IN ('fact','decision','person','concept') ORDER BY b DESC, importance DESC", scopes
    )

    sections = {
        "constraints": _fill(constraints, budgets["constraints"]),
        "preferences": _fill(prefs, budgets["preferences"]),
        "working": _fill(list(hub) + list(working), budgets["working"]),
    }
    parts = [HEADER]
    if project:
        parts.append(f"Active project: {project}\n")
    titles = {"constraints": "## Rules (always apply)", "preferences": "## Preferences",
              "working": "## Current context"}
    for key, lines in sections.items():
        if lines:
            parts.append(titles[key] + "\n" + "\n".join(lines) + "\n")
    ids = [line.rsplit("[", 1)[1].rstrip("]") for lines in sections.values() for line in lines]
    return {"project": project, "text": "\n".join(parts), "ids": ids,
            "empty": not any(sections.values())}
