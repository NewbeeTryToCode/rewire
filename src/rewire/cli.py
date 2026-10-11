"""rewire command line."""

import argparse
import json
import os
import sys

from . import __version__, integrations, ops, snapshot
from . import sync as sync_mod
from .brain import Brain, Locked, default_path
from .index import Index


def _brain(args) -> Brain:
    brain = Brain(args.brain)
    if not brain.exists:
        sys.exit(f"no brain at {brain.root} — run `rewire init` (or set REWIRE_BRAIN)")
    return brain


def _print(data, as_json: bool):
    if as_json:
        print(json.dumps(data, ensure_ascii=False, indent=1))
        return
    if isinstance(data, list):
        for item in data:
            if "error" in item:
                print(f"! {item['ref']}: {item['error']}")
            elif item.get("type") == "episode":
                excerpt = " / ".join(item["text"].splitlines())[:240]
                print(f"  ~ {item['date']} {item['session']}: {excerpt}")
            elif "body" in item:
                print(f"[{item['id'][-8:].lower()}] {item['type']} · {item['scope']} · importance {item['importance']}")
                print(f"  {item['title']}\n  {item['body']}\n")
            else:
                print(f"[{item['id'][-8:].lower()}] {item['score']:.2f} {item['type']:<10} {item['title']}")
    else:
        print(data)


def cmd_init(args):
    if args.from_url:
        brain = Brain.clone(args.from_url, args.brain, device=args.device)
        print(f"brain cloned from {args.from_url} to {brain.root} (device: {brain.device})")
        if not brain.config.sync_auto:
            print("auto sync is off; enable with `rewire remote set <url>`")
    else:
        brain = Brain.init(args.brain, device=args.device)
        print(f"brain created at {brain.root} (device: {brain.device})")
    print("next: `rewire install claude-code` and/or `rewire install codex`")


def cmd_prime(args):
    brain = _brain(args)
    index = Index(brain)
    index.refresh()
    snap = snapshot.build(brain, index, cwd=args.cwd or os.getcwd(), project=args.project)
    _print(snap if args.json else snap["text"], False if not args.json else True)


def cmd_recall(args):
    brain = _brain(args)
    index = Index(brain)
    index.refresh()
    hits = ops.recall(brain, index, " ".join(args.query), k=args.k, cwd=os.getcwd(), scope=args.scope,
                      episodes=args.episodes)
    _print(hits, args.json)


def cmd_get(args):
    brain = _brain(args)
    _print(ops.get(brain, args.ids, session=args.session or f"cli-{os.getppid()}", harness="cli"), args.json)


def cmd_remember(args):
    brain = _brain(args)
    text = " ".join(args.text)
    provenance = "user" if args.apply else "assistant"
    try:
        if args.apply:
            if not sys.stdin.isatty():
                sys.exit("--apply is only allowed from an interactive terminal (agents use the review queue)")
            node = ops.apply_change(brain, {"action": "NEW", "type": args.type, "text": text, "scope": args.scope,
                                            "importance": args.importance, "confidence": "explicit",
                                            "provenance": provenance})
            ops.write_home(brain, Index(brain))
            brain.commit(f"remember {node.id}")
            print(f"saved [{node.id[-8:].lower()}] {node.meta['title']}")
        else:
            inbox_id = ops.remember(brain, text, type_=args.type, scope=args.scope, provenance=provenance,
                                    source={"harness": "cli"})
            print(f"queued for review: {inbox_id} (approve with `rewire review approve {inbox_id}`)")
    except ops.OpError as exc:
        sys.exit(str(exc))


def cmd_consolidate(args):
    from .consolidate import Consolidator

    brain = _brain(args)
    log = (lambda *_: None) if args.quiet else (lambda m: print(m, file=sys.stderr))
    try:
        con = Consolidator(brain, log=log)
        if args.pending or not args.files:
            results = con.run_pending(extra_paths=args.files)
        else:
            results = [con.run_file(os.path.abspath(f), args.harness) for f in args.files]
    except Locked as exc:
        print(f"skipped: {exc}", file=sys.stderr)
        return
    if brain.config.sync_auto and args.pending:
        try:
            with brain.lock(timeout=120):
                log("sync: " + "; ".join(sync_mod.push(brain)))
        except (Locked, sync_mod.SyncError) as exc:
            log(f"sync: push skipped: {exc}")
    if args.json:
        print(json.dumps(results, ensure_ascii=False, indent=1))
        return
    for r in results:
        if r["status"] != "consolidated":
            continue
        print(f"{r['harness']} {r['session']} → project {r['project'] or '-'}")
        for a in r["applied"]:
            print(f"  + {a['action']:<10} {a['title']}")
        for q in r["queued"]:
            print(f"  ? {q['action']:<10} {q['title']}  (inbox {q['inbox_id']})")
        for x in r["rejected"]:
            print(f"  ✗ {x['title']} ({x['why']})")


def cmd_review(args):
    brain = _brain(args)
    if args.action == "list":
        items = list(brain.iter_inbox())
        if not items:
            print("inbox empty")
        for item in items:
            m = item.meta
            target = f" → {m['target']}" if m.get("target") else ""
            print(f"{m['inbox_id']}  {m.get('action', 'NEW')}{target}  [{m['type']} · {m['scope']} · "
                  f"{m['confidence']}/{m['provenance']}]\n    {item.body.strip()}\n    reason: {m.get('reason', '')}"
                  + (f"  flags: {m['flags']}" if m.get("flags") else ""))
        return
    if not args.ids:
        sys.exit("give inbox ids, or `all`")
    ids = [i.meta["inbox_id"] for i in brain.iter_inbox()] if args.ids == ["all"] else args.ids
    for inbox_id in ids:
        try:
            if args.action == "approve":
                node = ops.approve(brain, inbox_id)
                print(f"approved {inbox_id} → [{node.id[-8:].lower()}] {node.meta['title']}")
            else:
                ops.reject(brain, inbox_id)
                print(f"rejected {inbox_id}")
        except ops.OpError as exc:
            print(f"! {exc}", file=sys.stderr)
    ops.write_home(brain, Index(brain))
    brain.commit(f"review {args.action} {len(ids)} item(s)")


def cmd_rebuild(args):
    print(f"indexed {Index(_brain(args)).rebuild()} nodes")


def cmd_upgrade(args):
    brain = _brain(args)
    count = 0
    for node in brain.iter_nodes():
        brain.write_node(node.meta, node.body, path=node.path)
        count += 1
    Index(brain).refresh()
    ops.write_home(brain, Index(brain))
    brain.commit("upgrade node format")
    print(f"rewrote {count} nodes in the current format")


def cmd_doctor(args):
    print("\n".join(ops.doctor(_brain(args))))


def cmd_undo(args):
    try:
        print(f"reverted {ops.undo(_brain(args), args.session)}")
    except ops.OpError as exc:
        sys.exit(str(exc))


def cmd_remote(args):
    brain = _brain(args)
    if args.action == "set":
        if not args.url:
            sys.exit("usage: rewire remote set <git-url>")
        print("\n".join(sync_mod.set_remote(brain, args.url)))
    else:
        url = sync_mod.remote_url(brain)
        print(f"origin: {url or '(none)'}; auto sync: {'on' if brain.config.sync_auto else 'off'}; "
              f"device: {brain.device}")


def cmd_sync(args):
    brain = _brain(args)
    steps = {"pull": [sync_mod.pull], "push": [sync_mod.push]}.get(args.only, [sync_mod.pull, sync_mod.push])
    try:
        with brain.lock(timeout=120):
            for step in steps:
                for note in step(brain):
                    print(note)
    except (Locked, sync_mod.SyncError) as exc:
        sys.exit(f"sync failed: {exc}")


def cmd_mcp(args):
    from .mcp import serve

    serve(_brain(args))


def cmd_hook(args):
    brain = Brain(args.brain)
    try:
        payload = json.loads(sys.stdin.read() or "{}") if not sys.stdin.isatty() else {}
    except json.JSONDecodeError:
        payload = {}
    try:
        out = integrations.hook(brain, args.event, payload, harness=args.harness)
    except Exception as exc:  # a memory hook must never break the host session
        print(f"rewire hook error: {exc}", file=sys.stderr)
        return
    if out:
        print(out)


def cmd_install(args):
    if args.harness == "claude-code":
        print(integrations.install_claude_code(write=args.write))
    else:
        print(integrations.install_codex(args.skills_dir))


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="rewire", description="Own your brain, transfer it anywhere.")
    p.add_argument("--brain", default=default_path(), help="brain directory (default: $REWIRE_BRAIN or ~/brain)")
    p.add_argument("--version", action="version", version=__version__)
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("init", help="create a new brain, or attach to an existing one with --from")
    s.add_argument("--from", dest="from_url", metavar="URL", help="clone an existing brain repository")
    s.add_argument("--device", help="name of this machine in the event log (default: hostname)")
    s.set_defaults(func=cmd_init)

    s = sub.add_parser("prime", help="print the session-start memory snapshot")
    s.add_argument("--project")
    s.add_argument("--cwd")
    s.add_argument("--json", action="store_true")
    s.set_defaults(func=cmd_prime)

    s = sub.add_parser("recall", help="search memory")
    s.add_argument("query", nargs="+")
    s.add_argument("-k", type=int, default=8)
    s.add_argument("--episodes", type=int, default=3, help="raw transcript excerpts to include (0 = none)")
    s.add_argument("--scope", help="global | project:<slug> | all")
    s.add_argument("--json", action="store_true")
    s.set_defaults(func=cmd_recall)

    s = sub.add_parser("get", help="read memory notes by id")
    s.add_argument("ids", nargs="+")
    s.add_argument("--session")
    s.add_argument("--json", action="store_true")
    s.set_defaults(func=cmd_get)

    s = sub.add_parser("remember", help="propose a memory (queued for review)")
    s.add_argument("text", nargs="+")
    s.add_argument("--type", default="fact", choices=["constraint", "preference", "fact", "decision", "person", "concept"])
    s.add_argument("--scope", default="global")
    s.add_argument("--importance", type=int, default=3)
    s.add_argument("--apply", action="store_true", help="save immediately (interactive terminal only)")
    s.set_defaults(func=cmd_remember)

    s = sub.add_parser("consolidate", help="consolidate transcripts into memory")
    s.add_argument("files", nargs="*")
    s.add_argument("--pending", action="store_true", help="process pending + newly discovered transcripts")
    s.add_argument("--harness", choices=["claude-code", "codex"])
    s.add_argument("--quiet", action="store_true")
    s.add_argument("--json", action="store_true")
    s.set_defaults(func=cmd_consolidate)

    s = sub.add_parser("review", help="review the inbox")
    s.add_argument("action", nargs="?", default="list", choices=["list", "approve", "reject"])
    s.add_argument("ids", nargs="*")
    s.set_defaults(func=cmd_review)

    sub.add_parser("rebuild", help="rebuild the index").set_defaults(func=cmd_rebuild)
    sub.add_parser("doctor", help="health check").set_defaults(func=cmd_doctor)
    sub.add_parser("upgrade", help="rewrite all nodes in the current file format").set_defaults(func=cmd_upgrade)

    s = sub.add_parser("undo", help="revert a session's consolidation")
    s.add_argument("session")
    s.set_defaults(func=cmd_undo)

    s = sub.add_parser("remote", help="show or set the git remote used to sync this brain")
    s.add_argument("action", choices=["show", "set"], nargs="?", default="show")
    s.add_argument("url", nargs="?")
    s.set_defaults(func=cmd_remote)

    s = sub.add_parser("sync", help="pull and push the brain")
    s.add_argument("--only", choices=["pull", "push"])
    s.set_defaults(func=cmd_sync)

    sub.add_parser("mcp", help="run the MCP server on stdio").set_defaults(func=cmd_mcp)

    s = sub.add_parser("hook", help="harness lifecycle hook (reads hook JSON on stdin)")
    s.add_argument("event", choices=list(integrations.HOOK_EVENTS))
    s.add_argument("--harness", default="claude-code")
    s.set_defaults(func=cmd_hook)

    s = sub.add_parser("install", help="connect a harness")
    s.add_argument("harness", choices=["claude-code", "codex"])
    s.add_argument("--write", action="store_true", help="write Claude Code settings (backs up first)")
    s.add_argument("--skills-dir", help="Agent Skills directory for codex (default ~/.agents/skills)")
    s.set_defaults(func=cmd_install)
    return p


def main(argv=None):
    args = build_parser().parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
