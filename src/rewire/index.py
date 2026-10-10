"""Derived SQLite index: rebuildable at any time from nodes/ and events/."""

import json
import os
import re
import sqlite3

from . import activation

DDL = """
CREATE TABLE nodes(id TEXT PRIMARY KEY, type TEXT, title TEXT, body TEXT, scope TEXT,
                   importance INT, active INT, recorded_at TEXT, path TEXT, meta TEXT);
CREATE TABLE edges(src TEXT, dst TEXT, rel TEXT);
CREATE TABLE act(id TEXT PRIMARY KEY, b REAL);
CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, title, body, tokenize='unicode61 remove_diacritics 2');
"""


class Index:
    def __init__(self, brain):
        self.brain = brain
        self.path = brain.path(".index", "brain.db")
        self._db = None

    @property
    def db(self) -> sqlite3.Connection:
        if self._db is None:
            if not os.path.exists(self.path):
                self.rebuild()
            self._db = sqlite3.connect(self.path)
            self._db.row_factory = sqlite3.Row
        return self._db

    def close(self):
        if self._db is not None:
            self._db.close()
            self._db = None

    def rebuild(self) -> int:
        self.close()
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        tmp = self.path + ".tmp"
        if os.path.exists(tmp):
            os.remove(tmp)
        db = sqlite3.connect(tmp)
        db.executescript(DDL)
        nodes = self.brain.nodes()
        for n in nodes.values():
            m = n.meta
            db.execute(
                "INSERT INTO nodes VALUES (?,?,?,?,?,?,?,?,?,?)",
                (n.id, n.type, m["title"], n.body, m["scope"], m["importance"], int(n.active),
                 m["recorded_at"], n.path, json.dumps(m, ensure_ascii=False)),
            )
            db.execute("INSERT INTO fts VALUES (?,?,?)", (n.id, m["title"], n.body))
            for e in m.get("edges") or []:
                db.execute("INSERT INTO edges VALUES (?,?,?)", (n.id, e["to"], e["rel"]))
        acts = activation.compute(self.brain.iter_events(), nodes, self.brain.config.d)
        db.executemany("INSERT INTO act VALUES (?,?)", acts.items())
        db.commit()
        db.close()
        os.replace(tmp, self.path)
        return len(nodes)

    def refresh(self) -> None:
        """Cheap path after writes: rebuild (brains are small; keeps one code path)."""
        self.rebuild()

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
