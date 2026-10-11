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

from . import activation, fm
from .util import parse_iso

SCHEMA = "2"
ACT_MAX_AGE = 300  # seconds; activation ages in hours, so 5 minutes of staleness is invisible
SCAN_MAX_AGE = 60  # seconds between full stat() scans when no directory changed

DDL = """
CREATE TABLE meta(k TEXT PRIMARY KEY, v TEXT);
CREATE TABLE nodes(id TEXT PRIMARY KEY, type TEXT, title TEXT, body TEXT, scope TEXT,
                   importance INT, active INT, recorded_at TEXT, path TEXT, meta TEXT);
CREATE TABLE edges(src TEXT, dst TEXT, rel TEXT);
CREATE INDEX edges_src ON edges(src);
CREATE TABLE act(id TEXT PRIMARY KEY, b REAL);
CREATE TABLE files(path TEXT PRIMARY KEY, mtime_ns INT, size INT, id TEXT);
CREATE TABLE evfiles(path TEXT PRIMARY KEY, offset INT, tail TEXT);
CREATE TABLE uses(node TEXT, key TEXT, t TEXT, w REAL, PRIMARY KEY(node, key));
CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, title, body, tokenize='unicode61 remove_diacritics 2');
"""


class _Stale(Exception):
    """An event log was rewritten (e.g. git revert): fall back to a full rebuild."""


class Index:
    def __init__(self, brain):
        self.brain = brain
        self.path = brain.path(".index", "brain.db")
        self._db = None

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
        return db

    def close(self):
        if self._db is not None:
            self._db.close()
            self._db = None

    def __del__(self):
        self.close()

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
        db.execute("BEGIN IMMEDIATE")  # serialize concurrent refreshes (hook + MCP + consolidation)
        try:
            changed = self._sync_nodes(db)
            touched = self._sync_events(db)
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
                meta, body = fm.loads(fh.read())
            if old and old["id"]:
                self._delete_node(db, old["id"])
            node_id = meta.get("id")
            if node_id:
                if not fresh:
                    self._delete_node(db, node_id)
                active = int(not meta.get("retracted_at") and not meta.get("valid_to"))
                cur = db.execute(
                    "INSERT INTO nodes VALUES (?,?,?,?,?,?,?,?,?,?)",
                    (node_id, meta.get("type"), meta.get("title", ""), body, meta.get("scope", "global"),
                     meta.get("importance", 3), active, meta.get("recorded_at", ""), path,
                     json.dumps(meta, ensure_ascii=False)),
                )
                db.execute("INSERT INTO fts(rowid, id, title, body) VALUES (?,?,?,?)",
                           (cur.lastrowid, node_id, meta.get("title", ""), body))
                db.executemany("INSERT INTO edges VALUES (?,?,?)",
                               [(node_id, e["to"], e["rel"]) for e in meta.get("edges") or []
                                if isinstance(e, dict) and e.get("to")])
                changed.add(node_id)
            db.execute("INSERT OR REPLACE INTO files VALUES (?,?,?,?)", (path, mtime, size, node_id))
        return changed

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

    def search(self, query: str, scopes: list[str] | None = None, k: int = 8,
               beta: float | None = None, include_inactive: bool = False) -> list[dict]:
        terms = [t for t in re.findall(r"\w+", query.lower()) if len(t) > 1]
        if not terms:
            return []
        match = " OR ".join(f'"{t}"' for t in terms)
        sql = (
            "SELECT nodes.*, COALESCE(act.b,-20) AS b, bm25(fts, 3.0, 1.0) AS rank "
            "FROM fts JOIN nodes ON nodes.id = fts.id LEFT JOIN act ON act.id = nodes.id "
            "WHERE fts MATCH ?"
        )
        params: list = [match]
        if not include_inactive:
            sql += " AND nodes.active = 1"
        if scopes:
            sql += f" AND nodes.scope IN ({','.join('?' * len(scopes))})"
            params += scopes
        sql += " ORDER BY rank LIMIT ?"
        params.append(max(k * 4, 20))
        rows = self.db.execute(sql, params).fetchall()
        if not rows:
            return []
        beta = self.brain.config.recall_beta if beta is None else beta
        best = min(r["rank"] for r in rows)  # bm25: more negative is better
        worst = max(r["rank"] for r in rows)
        span = (worst - best) or 1.0
        out = []
        for r in rows:
            rel = 1.0 - (r["rank"] - best) / span * 0.9  # 0.1..1.0
            score = rel * (1.0 + beta * activation.squash(r["b"]))
            out.append({"id": r["id"], "type": r["type"], "title": r["title"], "scope": r["scope"],
                        "snippet": r["body"][:160], "score": round(score, 4), "b": round(r["b"], 3)})
        out.sort(key=lambda x: x["score"], reverse=True)
        return out[:k]
