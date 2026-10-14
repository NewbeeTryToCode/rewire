"""Derived SQLite index: rebuildable at any time from nodes/ and events/.

Kept fresh incrementally: only node files whose mtime/size changed are
re-parsed, only bytes appended to event logs are read, and activation is
recomputed for touched nodes (all nodes once it is older than ACT_MAX_AGE).
"""

import glob
import json
import math
import os
import re
import sqlite3
import time
from datetime import datetime, timezone

from . import activation, nodefmt, semantic, text
from .util import parse_iso

SCHEMA = "9"
# How much activation flows along each relation during recall (spreading activation).
REL_WEIGHT = {"part_of": 0.8, "depends_on": 0.8, "related_to": 0.6, "about": 0.5,
              "contradicts": 0.4, "supersedes": 0.2}
HOP_DECAY = 0.5
ACT_MAX_AGE = 300  # seconds; activation ages in hours, so 5 minutes of staleness is invisible
SCAN_MAX_AGE = 60  # seconds between full stat() scans when no directory changed

DDL = """
CREATE TABLE meta(k TEXT PRIMARY KEY, v TEXT);
CREATE TABLE nodes(id TEXT PRIMARY KEY, type TEXT, title TEXT, body TEXT, scope TEXT,
                   importance INT, active INT, recorded_at TEXT, path TEXT, meta TEXT, short TEXT);
CREATE INDEX nodes_short ON nodes(short);
CREATE TABLE edges(src TEXT, dst TEXT, rel TEXT);  -- src: full id, dst: 8-char id suffix
CREATE INDEX edges_src ON edges(src);
CREATE INDEX edges_dst ON edges(dst);
CREATE TABLE act(id TEXT PRIMARY KEY, b REAL);
CREATE TABLE files(path TEXT PRIMARY KEY, mtime_ns INT, size INT, id TEXT);
CREATE TABLE evfiles(path TEXT PRIMARY KEY, offset INT, tail TEXT);
CREATE TABLE uses(node TEXT, key TEXT, t TEXT, w REAL, PRIMARY KEY(node, key));
CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, title, body, keywords,
                                  tokenize='unicode61 remove_diacritics 2');
CREATE TABLE epfiles(path TEXT PRIMARY KEY, mtime_ns INT, size INT);
CREATE TABLE episodes(path TEXT, date TEXT, text TEXT, first INT, last INT, scope TEXT);  -- turn range; scope of the session
CREATE INDEX episodes_path ON episodes(path);
CREATE VIRTUAL TABLE fts_ep USING fts5(text, tokenize='unicode61 remove_diacritics 2');
CREATE TABLE vec(id TEXT PRIMARY KEY, v BLOB);
CREATE TABLE epvec(rid INTEGER PRIMARY KEY, v BLOB);
"""


_TURN = re.compile(r"\[(\d+) ")


class _Stale(Exception):
    """An event log was rewritten (e.g. git revert): fall back to a full rebuild."""



SUMMARY_CHARS = 600


def _session_scope(part: str, previous: str = "global") -> str:
    """Scope of the session one consolidation run belongs to ('global' when it had no project). A part
    without the header (a `---` rule inside a summary splits a run) keeps the scope of the part before it."""
    m = re.search(r"(?m)^- project: (.+?)[ \t]*$", part)
    if not m:
        return previous
    return f"project:{m.group(1)}" if m.group(1) != "-" else "global"


def _summary(part: str) -> str:
    """'Summary: …' row text of one consolidation run in a session file ('' if none)."""
    m = re.search(r"\n## Summary\n(.*?)(?:\n## |\Z)", part, re.S)
    body = " ".join(m.group(1).split()) if m else ""
    return f"Summary: {body[:SUMMARY_CHARS]}" if body and body != "-" else ""

class Index:
    def __init__(self, brain):
        self.brain = brain
        self.path = brain.path(".index", "brain.db")
        self._db = None
        self._mat = None  # cached (ids, scopes, active, matrix) for semantic search
        self._epmat = None

    @property
    def db(self) -> sqlite3.Connection:
        if self._db is None:
            self.refresh()
        return self._db

    def _connect(self) -> sqlite3.Connection:
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        db = sqlite3.connect(self.path, timeout=30, isolation_level=None)
        db.row_factory = sqlite3.Row
        db.execute("PRAGMA journal_mode=WAL")
        try:
            ok = db.execute("SELECT v FROM meta WHERE k='schema'").fetchone()
        except sqlite3.OperationalError:
            ok = None
        if not ok or ok[0] != SCHEMA:
            db.close()
            for suffix in ("", "-wal", "-shm"):
                if os.path.exists(self.path + suffix):
                    os.remove(self.path + suffix)
            db = sqlite3.connect(self.path, timeout=30, isolation_level=None)
            db.row_factory = sqlite3.Row
            db.execute("PRAGMA journal_mode=WAL")
            db.executescript(DDL)
            db.execute("INSERT INTO meta VALUES ('schema', ?)", (SCHEMA,))
            db.execute("INSERT INTO meta VALUES ('semantic', ?)", (str(int(semantic.available())),))
        return db

    def close(self):
        if self._db is not None:
            self._db.close()
            self._db = None

    def __del__(self):
        try:
            self.close()
        except Exception:  # e.g. collected on another thread; the OS reclaims the handle
            self._db = None

    def rebuild(self) -> int:
        self.close()
        for suffix in ("", "-wal", "-shm"):
            if os.path.exists(self.path + suffix):
                os.remove(self.path + suffix)
        self.refresh()
        return self.db.execute("SELECT count(*) FROM nodes").fetchone()[0]

    def refresh(self, max_age: float = ACT_MAX_AGE) -> None:
        if self._db is None:
            self._db = self._connect()
        db = self._db
        sem = db.execute("SELECT v FROM meta WHERE k='semantic'").fetchone()
        if sem and sem[0] != str(int(semantic.available())):  # extra installed/removed since the build
            self.rebuild()
            return
        db.execute("BEGIN IMMEDIATE")  # serialize concurrent refreshes (hook + MCP + consolidation)
        try:
            changed = self._sync_nodes(db)
            touched = self._sync_events(db)
            ep_changed = self._sync_episodes(db)
            if changed or ep_changed:
                self._embed_pending(db)
                self._mat = self._epmat = None
            row = db.execute("SELECT v FROM meta WHERE k='act_at'").fetchone()
            stale = not row or time.time() - float(row[0]) > max_age
            if stale:
                self._compute_act(db, None)
                db.execute("INSERT OR REPLACE INTO meta VALUES ('act_at', ?)", (str(time.time()),))
            elif changed or touched:
                self._compute_act(db, changed | touched)
            db.execute("COMMIT")
        except _Stale:
            db.execute("ROLLBACK")
            self.rebuild()
        except BaseException:
            db.execute("ROLLBACK")
            raise

    # ---- incremental sync ------------------------------------------------

    def _delete_node(self, db, node_id: str) -> None:
        row = db.execute("SELECT rowid FROM nodes WHERE id=?", (node_id,)).fetchone()
        if row:
            db.execute("DELETE FROM fts WHERE rowid=?", (row[0],))
        db.execute("DELETE FROM nodes WHERE id=?", (node_id,))
        db.execute("DELETE FROM edges WHERE src=?", (node_id,))
        db.execute("DELETE FROM act WHERE id=?", (node_id,))
        db.execute("DELETE FROM vec WHERE id=?", (node_id,))

    def _sync_nodes(self, db) -> set[str]:
        root = self.brain.path("nodes")
        if not os.path.isdir(root):
            return set()
        # rewire writes atomically (rename), which bumps the directory mtime; so if no
        # directory changed and a full scan ran recently, nothing needs re-reading.
        subdirs = [e for e in os.scandir(root) if e.is_dir()]
        stamp = ",".join(f"{e.name}:{e.stat().st_mtime_ns}" for e in sorted(subdirs, key=lambda e: e.name))
        row = db.execute("SELECT v FROM meta WHERE k='scan'").fetchone()
        if row:
            last_stamp, _, last_at = row[0].rpartition("@")
            if last_stamp == stamp and time.time() - float(last_at) < SCAN_MAX_AGE:
                return set()
        seen = {}
        for sub in subdirs:
            for entry in os.scandir(sub.path):
                if entry.name.endswith(".md"):
                    st = entry.stat()
                    seen[entry.path] = (st.st_mtime_ns, st.st_size)
        db.execute("INSERT OR REPLACE INTO meta VALUES ('scan', ?)", (f"{stamp}@{time.time()}",))
        known = {r["path"]: r for r in db.execute("SELECT * FROM files")}
        fresh = not known  # first build: nothing to delete, skip per-node cleanup queries
        changed = set()
        for path, row in known.items():
            if path not in seen:
                self._delete_node(db, row["id"])
                db.execute("DELETE FROM files WHERE path=?", (path,))
                changed.add(row["id"])
        for path, (mtime, size) in seen.items():
            old = known.get(path)
            if old and old["mtime_ns"] == mtime and old["size"] == size:
                continue
            with open(path, encoding="utf-8") as fh:
                meta, body = nodefmt.parse(fh.read())
            if old and old["id"]:
                self._delete_node(db, old["id"])
            node_id = meta.get("id")
            if node_id:
                if not fresh:
                    self._delete_node(db, node_id)
                active = int(not meta.get("retracted_at") and not meta.get("valid_to"))
                cur = db.execute(
                    "INSERT INTO nodes VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                    (node_id, meta.get("type"), meta.get("title", ""), body, meta.get("scope", "global"),
                     meta.get("importance", 3), active, meta.get("recorded_at", ""), path,
                     json.dumps(meta, ensure_ascii=False), node_id[-8:].lower()),
                )
                keywords = meta.get("keywords") or []
                db.execute("INSERT INTO fts(rowid, id, title, body, keywords) VALUES (?,?,?,?,?)",
                           (cur.lastrowid, node_id, meta.get("title", ""), body,
                            " ".join(keywords) if isinstance(keywords, list) else str(keywords)))
                db.executemany("INSERT INTO edges VALUES (?,?,?)",
                               [(node_id, e["to"], e["rel"]) for e in meta.get("edges") or []
                                if isinstance(e, dict) and e.get("to")])
                changed.add(node_id)
            db.execute("INSERT OR REPLACE INTO files VALUES (?,?,?,?)", (path, mtime, size, node_id))
        return changed

    EP_WINDOW, EP_STEP = 6, 4  # turns per episode chunk, and stride (chunks overlap by 2 turns)

    def _sync_episodes(self, db) -> None:
        seen = {}
        for path in glob.glob(self.brain.path("sessions", "*", "*.md")):
            st = os.stat(path)
            seen[path] = (st.st_mtime_ns, st.st_size)
        known = {r["path"]: r for r in db.execute("SELECT * FROM epfiles")}
        changed = False
        for path in set(known) - set(seen):
            self._delete_episodes(db, path)
            changed = True
        for path, (mtime, size) in seen.items():
            old = known.get(path)
            if old and old["mtime_ns"] == mtime and old["size"] == size:
                continue
            self._delete_episodes(db, path)
            changed = True
            with open(path, encoding="utf-8") as fh:
                content = fh.read()
            scope = "global"
            for part in content.split("\n---\n"):  # one file may hold several consolidation runs
                m = re.search(r"(?m)^- date: (\S+)", part)
                date = m.group(1) if m else os.path.basename(os.path.dirname(path))
                scope = _session_scope(part, scope)
                summary = _summary(part)
                if summary:  # the session's goal and outcome, searchable like an excerpt (first = last = 0)
                    cur = db.execute("INSERT INTO episodes VALUES (?,?,?,?,?,?)", (path, date, summary, 0, 0, scope))
                    db.execute("INSERT INTO fts_ep(rowid, text) VALUES (?,?)", (cur.lastrowid, summary))
                if "\n## Transcript\n" not in part:
                    continue
                lines = [l for l in part.split("\n## Transcript\n", 1)[1].splitlines() if l.startswith("[")]
                for start in range(0, max(len(lines) - 2, 1), self.EP_STEP):
                    part_lines = lines[start:start + self.EP_WINDOW]
                    chunk = "\n".join(part_lines)
                    if chunk:
                        turns = [int(m.group(1)) for m in map(_TURN.match, part_lines) if m]
                        cur = db.execute("INSERT INTO episodes VALUES (?,?,?,?,?,?)",
                                         (path, date, chunk, min(turns, default=0), max(turns, default=0), scope))
                        db.execute("INSERT INTO fts_ep(rowid, text) VALUES (?,?)", (cur.lastrowid, chunk))
            db.execute("INSERT OR REPLACE INTO epfiles VALUES (?,?,?)", (path, mtime, size))
        return changed

    def _embed_pending(self, db) -> None:
        """Vectors for nodes/episodes that have none yet (derived data, like everything here)."""
        if not semantic.available():
            return
        rows = db.execute("SELECT id, title, body, meta FROM nodes WHERE id NOT IN (SELECT id FROM vec)").fetchall()
        for start in range(0, len(rows), 512):
            part = rows[start:start + 512]
            texts = []
            for r in part:
                kw = json.loads(r["meta"]).get("keywords") or []
                texts.append(f"{r['title']}. {r['body']} {' '.join(kw) if isinstance(kw, list) else ''}")
            vecs = semantic.embed(texts)
            db.executemany("INSERT OR REPLACE INTO vec VALUES (?,?)",
                           [(r["id"], v.tobytes()) for r, v in zip(part, vecs)])
        eps = db.execute("SELECT rowid, text FROM episodes WHERE rowid NOT IN (SELECT rid FROM epvec)").fetchall()
        for start in range(0, len(eps), 512):
            part = eps[start:start + 512]
            vecs = semantic.embed([r["text"] for r in part])
            db.executemany("INSERT OR REPLACE INTO epvec VALUES (?,?)",
                           [(r["rowid"], v.tobytes()) for r, v in zip(part, vecs)])

    def _delete_episodes(self, db, path: str) -> None:
        for (rowid,) in db.execute("SELECT rowid FROM episodes WHERE path=?", (path,)).fetchall():
            db.execute("DELETE FROM fts_ep WHERE rowid=?", (rowid,))
            db.execute("DELETE FROM epvec WHERE rid=?", (rowid,))
        db.execute("DELETE FROM episodes WHERE path=?", (path,))
        db.execute("DELETE FROM epfiles WHERE path=?", (path,))

    def search_episodes(self, query: str, k: int = 5, seeds: list[dict] | None = None,
                        window: tuple[str, str] | None = None, scopes: list[str] | None = None) -> list[dict]:
        """Raw transcript excerpts (episodic memory), hybrid-ranked like memories.

        `scopes` limits excerpts to sessions of those scopes, as for memories (None: every session).

        `seeds` are memory hits for the same query: the excerpt each one was extracted from
        (its source session and turn) is boosted, so a well-matched fact brings its context.
        """
        limit = max(k * 4, 20)
        match = text.match_expression(query)
        cond = f" AND e.scope IN ({','.join('?' * len(scopes))})" if scopes else ""
        fts = []
        if match:
            fts = [r[0] for r in self.db.execute(  # ties broken by file and turn: identical in every implementation
                "SELECT fts_ep.rowid FROM fts_ep JOIN episodes e ON e.rowid = fts_ep.rowid WHERE fts_ep MATCH ?"
                f"{cond} ORDER BY bm25(fts_ep), e.path, e.first LIMIT ?", (match, *(scopes or []), limit))]
        sem = []
        if semantic.available():
            if self._epmat is None:
                import numpy as np

                rows = self.db.execute(
                    "SELECT rid, v, e.scope FROM epvec JOIN episodes e ON e.rowid = epvec.rid ORDER BY rid").fetchall()
                self._epmat = ([r["rid"] for r in rows], [r["scope"] for r in rows],
                               np.frombuffer(b"".join(r["v"] for r in rows), dtype=np.float32).reshape(len(rows), -1)
                               if rows else None)
            rids, ep_scopes, mat = self._epmat
            if mat is not None:
                sims = mat @ semantic.embed([query])[0]
                sem = [rids[i] for i in sims.argsort()[::-1] if not scopes or ep_scopes[i] in scopes][:limit]
        fused: dict[int, float] = {}
        for ranks in (fts, sem):
            for rank, rid in enumerate(ranks):
                fused[rid] = fused.get(rid, 0.0) + 1.0 / (self.RRF_K + rank + 1)
        scale = max(fused.values(), default=1.0 / (self.RRF_K + 1))  # bonus relative to the best match
        seeded = self._seed_episodes(seeds or [])
        for rid, bonus in seeded.items():
            fused[rid] = fused.get(rid, 0.0) + bonus * scale
        window = window or text.time_window(query)
        rows = {r["rid"]: r for r in self.db.execute(
            f"SELECT rowid AS rid, path, date, text, first, last FROM episodes e WHERE rowid IN ({','.join('?' * len(fused))})"
            f"{cond}", list(fused) + (scopes or []))} if fused else {}
        fused = {rid: score for rid, score in fused.items() if rid in rows}  # seeds may point out of scope
        if window:
            for rid, r in rows.items():
                if window[0] <= (r["date"] or "")[:10] <= window[1]:
                    fused[rid] += 1.0 / self.RRF_K
        best = sorted(fused, key=lambda x: (-fused[x], rows[x]["path"], rows[x]["first"]) if x in rows else (0, "", 0))[:k]
        best, texts = self._merge(best, rows)
        for pos, rid in enumerate(best):
            if pos >= self.EP_WHOLE and rows[rid]["first"] != 0 and rid not in seeded:
                texts[rid] = self._trim(query, texts[rid])
        out = [{"id": f"ep:{rid}", "type": "episode", "date": rows[rid]["date"],
                "session": os.path.relpath(rows[rid]["path"], self.brain.root), "text": texts[rid]}
               for rid in best]
        if out and rows[best[0]]["first"] != 0:
            # the top excerpt carries its session's goal and outcome (bookends), unless it is the summary itself
            summary = " / ".join(r[0] for r in self.db.execute(
                "SELECT text FROM episodes WHERE path = ? AND first = 0 AND last = 0 ORDER BY rowid",
                (rows[best[0]]["path"],)))
            if summary:
                out[0]["summary"] = summary[:300]
        return out

    # Excerpts below the top EP_WHOLE keep only the turns that share a content word with the query, plus
    # EP_CONTEXT turns either side: 11% fewer excerpt tokens on LoCoMo, and at the same token budget
    # (more excerpts) +1 point of evidence recall (experiments/eval/results/2026-10-14-locomo-trim.md).
    EP_WHOLE, EP_CONTEXT = 2, 1
    _SPEAKER = re.compile(r"^\[[^]]*\] (?:\w+: )?")

    @classmethod
    def _trim(cls, query: str, excerpt: str) -> str:
        lines = excerpt.splitlines()
        hits = [i for i, line in enumerate(lines) if text.overlap(query, cls._SPEAKER.sub("", line)) > 0]
        if not hits:  # matched through vectors or a time window: no word to anchor on, keep it whole
            return excerpt
        keep = sorted({j for i in hits for j in range(i - cls.EP_CONTEXT, i + cls.EP_CONTEXT + 1) if 0 <= j < len(lines)})
        return "\n".join(lines[j] for j in keep)

    @staticmethod
    def _merge(best: list[int], rows: dict) -> tuple[list[int], dict[int, str]]:
        """Excerpts of one session whose turn ranges overlap or touch become one excerpt (no line twice), at
        the rank of the best of them. Windows overlap by two turns, so without this the same lines are sent
        twice: 11-12% of the excerpt tokens on LoCoMo, with the same evidence (experiments/eval/results)."""
        groups: list[list] = []  # [anchor rowid, path, first turn, last turn, {turn: line}]
        for rid in best:
            r = rows.get(rid)
            if r is None:
                continue
            lines: dict[int, str] = {}
            for line in r["text"].splitlines():
                m = _TURN.match(line)
                lines[int(m.group(1)) if m else -len(lines) - 1] = line
            lo, hi = r["first"], r["last"]
            for g in groups:  # a session summary (turn 0) is never merged; neither are runs that reuse turn numbers
                if (lo > 0 and g[2] > 0 and g[1] == r["path"] and lo <= g[3] + 1 and hi >= g[2] - 1
                        and all(g[4].get(n, line) == line for n, line in lines.items())):
                    g[2], g[3] = min(g[2], lo), max(g[3], hi)
                    g[4].update(lines)
                    break
            else:
                groups.append([rid, r["path"], lo, hi, lines])
        return [g[0] for g in groups], {g[0]: "\n".join(g[4][n] for n in sorted(g[4])) for g in groups}

    SEED_WEIGHT = 0.35  # source-excerpt bonus for the top memory hit, as a share of the best excerpt's score (tuned: experiments/locomo/evidence.py)

    def _seed_episodes(self, seeds: list[dict]) -> dict[int, float]:
        """Episode rowid -> bonus, for the excerpts the seed memories were extracted from."""
        from .util import slugify

        out: dict[int, float] = {}
        if not seeds or self.SEED_WEIGHT <= 0:
            return out
        top = max(h.get("score", 0) for h in seeds) or 1.0
        metas = {r["id"]: r["meta"] for r in self.db.execute(
            f"SELECT id, meta FROM nodes WHERE id IN ({','.join('?' * len(seeds))})", [h["id"] for h in seeds])}
        for h in seeds:
            src = nodefmt.parse_source(json.loads(metas.get(h["id"]) or "{}").get("source"))
            if not src:
                continue
            harness, session, turn = src
            name = f"{harness}-{slugify(session, 40)}.md"
            bonus = self.SEED_WEIGHT * h.get("score", 0) / top
            for r in self.db.execute("SELECT rowid AS rid FROM episodes WHERE path LIKE ? AND first <= ? AND last >= ?",
                                     ("%" + os.sep + name, turn, turn)):
                out[r["rid"]] = out.get(r["rid"], 0.0) + bonus
        return out

    def _sync_events(self, db) -> set[str]:
        touched = set()
        known = {r["path"]: r for r in db.execute("SELECT * FROM evfiles")}
        for path in glob.glob(self.brain.path("events", "*", "*.jsonl")):
            size = os.path.getsize(path)
            row = known.get(path)
            offset = row["offset"] if row else 0
            with open(path, "rb") as fh:
                if row:
                    if size < offset:
                        raise _Stale(path)
                    tail = row["tail"] or ""
                    if tail:
                        fh.seek(offset - len(tail.encode("latin-1")))
                        if fh.read(len(tail.encode("latin-1"))).decode("latin-1") != tail:
                            raise _Stale(path)
                    if size == offset:
                        continue
                fh.seek(offset)
                chunk = fh.read()
            end = chunk.rfind(b"\n") + 1  # only complete lines; a torn last line is read next time
            if end == 0:
                continue
            rows = []
            for line in chunk[:end].decode("utf-8", errors="replace").splitlines():
                try:
                    ev = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if ev.get("ev") not in activation.USE_EVENTS or not ev.get("node") or not ev.get("t"):
                    continue
                rows.append((ev["node"], ev.get("session") or ev["t"], ev["t"], float(ev.get("w", 1.0))))
                touched.add(ev["node"])
            db.executemany(
                "INSERT INTO uses VALUES (?,?,?,?) ON CONFLICT(node, key) DO UPDATE "
                "SET t=excluded.t, w=excluded.w WHERE excluded.w > uses.w", rows)
            new_offset = offset + end
            tail = chunk[max(0, end - 64):end].decode("latin-1")
            db.execute("INSERT OR REPLACE INTO evfiles VALUES (?,?,?)", (path, new_offset, tail))
        return touched

    def _compute_act(self, db, ids: set[str] | None) -> None:
        d = self.brain.config.d
        now = datetime.now(timezone.utc)
        if ids is None:
            db.execute("DELETE FROM act")
            nodes = db.execute("SELECT id, recorded_at FROM nodes").fetchall()
            uses = db.execute("SELECT node, t, w FROM uses").fetchall()
        else:
            ids = list(ids)
            nodes, uses = [], []
            for i in range(0, len(ids), 500):
                part = ids[i:i + 500]
                marks = ",".join("?" * len(part))
                nodes += db.execute(f"SELECT id, recorded_at FROM nodes WHERE id IN ({marks})", part).fetchall()
                uses += db.execute(f"SELECT node, t, w FROM uses WHERE node IN ({marks})", part).fetchall()
        now_ts = now.timestamp()
        total = {}

        def add(node_id, when, w):
            age_h = max((now_ts - parse_iso(when).timestamp()) / 3600.0, activation.MIN_AGE_H)
            total[node_id] = total.get(node_id, 0.0) + w * age_h ** (-d)

        present = {r["id"] for r in nodes}
        for r in nodes:
            if r["recorded_at"]:
                add(r["id"], r["recorded_at"], 1.0)
        for r in uses:
            if r["node"] in present:
                add(r["node"], r["t"], r["w"])
        db.executemany("INSERT OR REPLACE INTO act VALUES (?,?)",
                       [(i, math.log(v) if v > 0 else activation.FLOOR) for i, v in total.items()])

    # ---- queries ---------------------------------------------------------

    def rows(self, where: str = "1", params=()) -> list[sqlite3.Row]:
        return self.db.execute(
            f"SELECT nodes.*, COALESCE(act.b, -20) AS b FROM nodes LEFT JOIN act USING(id) WHERE {where}",
            params,
        ).fetchall()

    def neighbors(self, node_id: str) -> list[tuple[str, str]]:
        """(neighbor full id, relation) in both directions."""
        short = node_id[-8:].lower()
        out = self.db.execute(
            "SELECT n.id, e.rel FROM edges e JOIN nodes n ON n.short = e.dst WHERE e.src = ? "
            "UNION ALL SELECT e.src, e.rel FROM edges e WHERE e.dst = ?", (node_id, short)).fetchall()
        return [(r[0], r[1]) for r in out if r[0] != node_id]

    # ---- hybrid retrieval ------------------------------------------------

    RRF_K = 60  # reciprocal-rank-fusion constant (standard value)
    # The bundled static embeddings are much weaker than BM25 + keywords on their own (R@5 57% vs 90%),
    # so they only add a small bonus; tuned in experiments/retrieval, validated on LoCoMo.
    SEM_WEIGHT = 0.15
    SEM_TOP = 10

    def _matrix(self):
        """(ids, scopes, matrix) of node vectors, cached until the index changes."""
        if self._mat is None:
            import numpy as np

            rows = self.db.execute("SELECT nodes.id, nodes.scope, vec.v FROM vec JOIN nodes USING(id)").fetchall()
            if not rows:
                self._mat = ([], [], None)
            else:
                mat = np.frombuffer(b"".join(r["v"] for r in rows), dtype=np.float32).reshape(len(rows), -1)
                self._mat = ([r["id"] for r in rows], [r["scope"] for r in rows], mat)
        return self._mat

    def _semantic_ranks(self, query: str, scopes, limit: int) -> list[str]:
        if not semantic.available():
            return []
        ids, node_scopes, mat = self._matrix()
        if mat is None:
            return []
        qv = semantic.embed([query])[0]
        sims = mat @ qv
        order = sims.argsort()[::-1]
        allowed = set(scopes) if scopes else None
        out = []
        for i in order:
            if allowed is None or node_scopes[i] in allowed:
                out.append(ids[i])
                if len(out) >= limit:
                    break
        return out

    def search(self, query: str, scopes: list[str] | None = None, k: int = 8, beta: float | None = None,
               include_inactive: bool = False, spread: bool = True, history: bool | None = None,
               window: tuple[str, str] | None = None) -> list[dict]:
        """Hybrid recall: BM25 + semantic vectors (if installed), fused by rank, then spread over the graph.

        - Query time intent ("in July 2023") boosts memories dated in that window.
        - Superseded memories hand their activation to their replacement; they are returned
          themselves (marked "superseded") only for history questions ("used to", "first", "dulu").
        """
        limit = max(k * 4, 20)
        match = text.match_expression(query)
        fts_ids: list[str] = []
        info: dict = {}
        if match:
            sql = ("SELECT nodes.*, COALESCE(act.b,-20) AS b, bm25(fts, 3.0, 1.0, 2.0) AS rank FROM fts "
                   "JOIN nodes ON nodes.rowid = fts.rowid LEFT JOIN act ON act.id = nodes.id WHERE fts MATCH ?")
            params: list = [match]
            if scopes:
                sql += f" AND nodes.scope IN ({','.join('?' * len(scopes))})"
                params += scopes
            sql += " ORDER BY bm25(fts, 3.0, 1.0, 2.0), nodes.id LIMIT ?"
            params.append(limit)
            for r in self.db.execute(sql, params):
                fts_ids.append(r["id"])
                info[r["id"]] = r
        sem_ids = self._semantic_ranks(query, scopes, self.SEM_TOP) if self.SEM_WEIGHT > 0 else []
        fused: dict[str, float] = {i: 0.0 for i in fts_ids + sem_ids}
        window = window or text.time_window(query)  # an explicit window comes from the calling model
        if window:  # dated memories inside the asked window join the candidates
            for r in self.db.execute(
                    "SELECT nodes.*, COALESCE(act.b,-20) AS b FROM nodes LEFT JOIN act USING(id) "
                    "WHERE json_extract(meta, '$.valid_from') BETWEEN ? AND ? LIMIT 200", window):
                if not scopes or r["scope"] in scopes:
                    info.setdefault(r["id"], r)
                    fused.setdefault(r["id"], 0.0)
        if not fused:
            return []
        self._fetch(info, [i for i in fused if i not in info])
        # BM25 keeps its magnitudes (they carry real signal); the semantic ranker adds a bonus
        # that decays with its rank. Plain RRF flattened BM25 and hurt MRR (experiments/retrieval).
        ranks = {i: info[i]["rank"] for i in fts_ids}
        best, worst = (min(ranks.values()), max(ranks.values())) if ranks else (0.0, 0.0)
        span = (worst - best) or 1.0
        rel = {i: (1.0 - (ranks[i] - best) / span * 0.9) if i in ranks else 0.0 for i in fused}
        for pos, i in enumerate(sem_ids):
            rel[i] += self.SEM_WEIGHT * (1.0 - pos / max(len(sem_ids), 1))
        top = max(rel.values()) or 1.0
        rel = {i: max(0.1, v / top) for i, v in rel.items()}
        if window:
            for i in rel:
                vf = json.loads(info[i]["meta"]).get("valid_from") or ""
                if window[0] <= vf[:10] <= window[1]:
                    rel[i] += 0.5
            top = max(rel.values())
            rel = {i: v / top for i, v in rel.items()}
        history = text.history_intent(query) if history is None else history
        beta = self.brain.config.recall_beta if beta is None else beta
        energy = dict(rel)
        via: dict[str, str] = {}
        if spread:
            frontier = sorted(rel.items(), key=lambda x: (-x[1], x[0]))[:10]
            for hop in (1, 2):
                nxt: dict[str, float] = {}
                for src, amount in frontier:
                    links = self.neighbors(src)
                    if not links:
                        continue
                    fan = 1.0 + math.log(1 + len(links))
                    src_inactive = src in info and not info[src]["active"]
                    for dst, relation in links:
                        if src_inactive and relation == "supersedes":
                            gain = amount * 0.9  # an outdated memory points straight at its replacement
                        else:
                            gain = amount * REL_WEIGHT.get(relation, 0.3) * HOP_DECAY ** hop / fan
                        if gain < 0.02:
                            continue
                        nxt[dst] = nxt.get(dst, 0.0) + gain
                        if dst not in rel and dst not in via:
                            via[dst] = (f"replaces outdated: {info[src]['title']}" if src_inactive else
                                        f"{relation} ↔ {info[src]['title'] if src in info else src[-8:].lower()}")
                for dst, gain in nxt.items():
                    energy[dst] = energy.get(dst, 0.0) + gain
                frontier = sorted(nxt.items(), key=lambda x: (-x[1], x[0]))[:10]
            self._fetch(info, [i for i in energy if i not in info], scopes)
        out = []
        for node_id, e in energy.items():
            r = info.get(node_id)
            if r is None or (r["type"] == "project" and node_id not in rel):
                continue  # out of scope / hubs reached only through links
            status = "active" if r["active"] else "superseded"
            if status != "active" and not (include_inactive or history):
                continue
            score = e * (1.0 + beta * activation.squash(r["b"])) * (1.0 if status == "active" else 0.6)
            meta = json.loads(r["meta"])
            item = {"id": node_id, "type": r["type"], "title": r["title"], "scope": r["scope"],
                    "snippet": r["body"][:160], "score": round(score, 4), "b": round(r["b"], 3),
                    "date": (meta.get("valid_from") or "")[:10], "status": status}
            if node_id in via:
                item["via"] = via[node_id]
            out.append(item)
        out.sort(key=lambda x: (-x["score"], x["id"]))  # ties by id: deterministic across implementations
        return out[:k]

    def _fetch(self, info: dict, ids: list[str], scopes=None) -> None:
        for chunk in range(0, len(ids), 500):
            part = ids[chunk:chunk + 500]
            cond = f" AND nodes.scope IN ({','.join('?' * len(scopes))})" if scopes else ""
            for r in self.db.execute(
                    f"SELECT nodes.*, COALESCE(act.b,-20) AS b FROM nodes LEFT JOIN act USING(id) "
                    f"WHERE id IN ({','.join('?' * len(part))}){cond}", part + (scopes or [])):
                info[r["id"]] = r
