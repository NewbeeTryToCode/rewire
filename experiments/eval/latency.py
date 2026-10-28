"""Recall latency of a running rewire MCP server (warm: index and model loaded), on the brains of a run.py run.

    python experiments/eval/latency.py OUT/results-<bench>-<label>.json [more results files] [--repeat 3]

For every brain in the results files, one `rewire mcp` server is started (with the brain's frozen clock) and
asked, one call at a time, the same recall each question made in the run: query, k, episodes, and the extra
queries (`also`) it was given. The first call per server is reported apart (it loads the index). Nothing else
should run on the machine meanwhile: the numbers are what an agent waits for per recall over MCP.
"""

import argparse
import json
import os
import platform
import subprocess
import sys
import time
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import rewire_cli as rw  # noqa: E402
from run import frozen_now, pct  # noqa: E402


class Server:
    def __init__(self, root: str):
        env = dict(os.environ, REWIRE_NOW=frozen_now(root))
        self.proc = subprocess.Popen([rw.BIN, "--brain", root, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL, text=True, env=env)
        self.next_id = 0
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "latency.py", "version": "1"}})
        self.notify("notifications/initialized")

    def notify(self, method: str) -> None:
        self.proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": method}) + "\n")
        self.proc.stdin.flush()

    def request(self, method: str, params: dict) -> dict:
        self.next_id += 1
        self.proc.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params}) + "\n")
        self.proc.stdin.flush()
        reply = json.loads(self.proc.stdout.readline())
        if "error" in reply:
            raise RuntimeError(reply["error"])
        return reply["result"]

    def recall(self, args: dict) -> float:
        start = time.perf_counter()
        result = self.request("tools/call", {"name": "recall", "arguments": args})
        ms = (time.perf_counter() - start) * 1000
        if result.get("isError"):
            raise RuntimeError(result["content"][0]["text"])
        return ms

    def close(self) -> None:
        self.proc.stdin.close()
        self.proc.wait(timeout=30)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("results", nargs="+")
    ap.add_argument("--repeat", type=int, default=3, help="times each question's recall is timed")
    ap.add_argument("--out", help="write the summary JSON here")
    args = ap.parse_args()
    rw.check()
    by_bench: dict[str, list[float]] = defaultdict(list)
    first_calls: dict[str, list[float]] = defaultdict(list)
    for path in args.results:
        data = json.load(open(path))
        s = data["summary"]
        by_brain = defaultdict(list)
        for row in data["rows"]:
            if row.get("brain"):
                by_brain[row["brain"]].append(row)
        if not by_brain:
            sys.exit(f"{path}: rows carry no brain path (run.py from before latency.py, or --mode fullctx)")
        bench = f"{s['bench']} ({os.path.basename(path)})"
        for root, rows in sorted(by_brain.items()):
            server = Server(root)
            try:
                first_calls[bench].append(server.recall({"query": "index warm up", "k": 1, "episodes": 1, "scope": "all"}))
                for _ in range(args.repeat):
                    for row in rows:
                        call = {"query": row["question"], "k": s["k"], "episodes": s["episodes"], "scope": "all"}
                        if row.get("also"):
                            call["also"] = row["also"]
                        by_bench[bench].append(server.recall(call))
            finally:
                server.close()
    summary = {"machine": {"os": platform.platform(), "cpu": platform.processor() or platform.machine(),
                           "cores": os.cpu_count()}, "rewire": rw.version(), "repeat": args.repeat, "benches": {}}
    print("| run | recalls | p50 ms | p95 ms | p99 ms | first call per server p50 ms |\n|---|---|---|---|---|---|")
    for bench, ms in by_bench.items():
        row = {"recalls": len(ms), "p50": pct(ms, 0.5), "p95": pct(ms, 0.95), "p99": pct(ms, 0.99),
               "first_call_p50": pct(first_calls[bench], 0.5)}
        summary["benches"][bench] = row
        print(f"| {bench} | {row['recalls']} | {row['p50']:.1f} | {row['p95']:.1f} | {row['p99']:.1f} | "
              f"{row['first_call_p50']:.0f} |")
    print(f"\n{summary['rewire']} · {summary['machine']['cpu']} ({summary['machine']['cores']} cores) · "
          f"{summary['machine']['os']} · one call at a time, warm server")
    if args.out:
        json.dump(summary, open(args.out, "w"), indent=1)


if __name__ == "__main__":
    main()
