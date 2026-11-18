#!/usr/bin/env python3
"""Knowledge-graph smoke test for the stdio server, on the throwaway test account.

Usage:
    set -a; source .env; set +a
    YAP_USER_EMAIL=yap-mcp-test@popovit.ch python3 yap-mcp/smoke/stdio_knowledge.py target/debug/yap-mcp

Builds a small graph (fresh ids each run, since ids are never reused), reviews
it, splits a concept, links a document, then restarts the server and checks
that the graph folds back identically from Supabase.
"""
import json
import os
import subprocess
import sys
import threading
import uuid

BIN = sys.argv[1]
RUN = uuid.uuid4().hex[:6]


class Server:
    def __init__(self):
        self.proc = subprocess.Popen(
            [BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, env=os.environ,
        )
        threading.Thread(target=self.pump_stderr, daemon=True).start()
        self.next_id = 0
        self.send("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                 "clientInfo": {"name": "smoke-knowledge", "version": "0"}})
        self.send("notifications/initialized", notification=True)

    def pump_stderr(self):
        for line in self.proc.stderr:
            sys.stderr.write("[server] " + line)

    def send(self, method, params=None, notification=False):
        msg = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        if not notification:
            self.next_id += 1
            msg["id"] = self.next_id
        self.proc.stdin.write(json.dumps(msg) + "\n")
        self.proc.stdin.flush()
        if notification:
            return None
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise SystemExit("server closed stdout")
            resp = json.loads(line)
            if resp.get("id") == self.next_id:
                return resp

    def tool(self, name, arguments, want_error=False):
        resp = self.send("tools/call", {"name": name, "arguments": arguments})
        result = resp.get("result", resp.get("error"))
        is_error = isinstance(result, dict) and result.get("isError", False)
        text = "\n".join(b["text"] for b in result.get("content", []) if b.get("type") == "text")
        print(f"\n=== {name} (error={is_error}) ===\n{text[:1500]}")
        assert is_error == want_error, f"{name}: expected error={want_error}"
        return text if is_error else json.loads(text)

    def close(self):
        self.proc.stdin.close()
        self.proc.wait()


def cid(name):
    return f"{name}-{RUN}"


s = Server()
s.tool("knowledge_apply", {"ops": [
    {"op": "add_concept", "id": cid("add"), "title": "Adding numbers",
     "tags": [cid("arithmetic")]},
    {"op": "add_concept", "id": cid("multiply"), "title": "Multiplying numbers"},
    {"op": "tag", "concepts": [cid("add"), cid("multiply")], "tags": [cid("for-school")]},
    {"op": "set_link", "advanced": cid("multiply"), "basic": cid("add"),
     "prerequisite": True, "encompassing": 0.8},
]})
# A cycle is rejected, and nothing in the batch is applied.
s.tool("knowledge_apply", {"ops": [
    {"op": "add_concept", "id": cid("never"), "title": "Never added"},
    {"op": "set_link", "advanced": cid("add"), "basic": cid("multiply"),
     "prerequisite": True, "encompassing": 0},
]}, want_error=True)

nxt = s.tool("knowledge_next", {})
assert cid("add") in [c["id"] for c in nxt["frontier"]]
assert cid("multiply") not in [c["id"] for c in nxt["frontier"]]

# Tags: counted in the graph, and they scope a session.
graph = s.tool("knowledge_graph", {"tag": cid("for-school")})
assert {c["id"] for c in graph["concepts"]} == {cid("add"), cid("multiply")}, graph["concepts"]
counts = {t["tag"]: t["concepts"] for t in graph["tags"]}
assert counts[cid("for-school")] == 2 and counts[cid("arithmetic")] == 1, counts
assert [c["id"] for c in s.tool("knowledge_next", {"tag": cid("arithmetic")})["frontier"]] == [cid("add")]
assert s.tool("knowledge_next", {"tag": cid("nothing-here")})["frontier"] == []

out = s.tool("knowledge_apply", {"ops": [
    {"op": "review", "id": cid("add"), "score": 1.0, "question": "2+3?", "answer": "5",
     "notes": "instant", "extra_credit": "Asked 3+2 too; said 5, so order doesn't matter."},
    {"op": "review", "id": cid("multiply"), "score": 0.3, "question": "4*6?", "answer": "20",
     "notes": "confused with 4*5"},
]})
assert out["remediation"][0]["failed"] == cid("multiply")
assert out["remediation"][0]["prerequisites"][0]["id"] == cid("add")

# A failed first attempt leaves the concept unlearned, still on the frontier,
# and the planner carries its past questions so no per-concept call is needed.
nxt = s.tool("knowledge_next", {})
mine = [c for c in nxt["frontier"] if c["id"] == cid("multiply")]
assert mine and mine[0]["status"] == "new", nxt["frontier"]
assert mine[0]["recent_questions"][0]["question"] == "4*6?"
assert "description" in mine[0] and "documents" in mine[0]

out = s.tool("knowledge_apply", {"ops": [
    {"op": "split_concept", "id": cid("multiply"), "into": [
        {"id": cid("times-tables"), "title": "Times tables"},
        {"id": cid("long-multiplication"), "title": "Long multiplication"},
    ]},
    {"op": "set_known", "id": cid("times-tables"), "known": True},
]})

docs_ok = s.tool("knowledge_save_document", {
    "id": cid("arithmetic-notes"), "title": "Arithmetic notes",
    "body": "Multiplication is repeated addition.",
}, want_error=os.environ.get("SMOKE_NO_DOCUMENTS") == "1")
if os.environ.get("SMOKE_NO_DOCUMENTS") != "1":
    s.tool("knowledge_apply", {"ops": [
        {"op": "set_document_link", "document": cid("arithmetic-notes"),
         "concept": cid("multiply"), "linked": True, "note": "the definition"},
    ]})

concept = s.tool("knowledge_concept", {"id": cid("multiply")})
assert concept["split_into"] == [cid("times-tables"), cid("long-multiplication")]
assert concept["review_history"][0]["answer"] == "20"
if os.environ.get("SMOKE_NO_DOCUMENTS") != "1":
    assert concept["documents"][0]["body"] == "Multiplication is repeated addition."

reviews = s.tool("knowledge_reviews", {"limit": 5})
assert {r["concept"] for r in reviews["reviews"][:2]} == {cid("add"), cid("multiply")}

before = s.tool("knowledge_graph", {})
s.close()

# Extra credit is kept with the review and survives a refold.
extra = lambda: s.tool("knowledge_concept", {"id": cid("add")})["review_history"][0]["extra_credit"]

# A fresh process folds the same graph from Supabase.
s = Server()
after = s.tool("knowledge_graph", {})
mine = lambda g: sorted((c["id"], c["status"], c["reviews"]) for c in g["concepts"] if RUN in c["id"])
assert mine(before) == mine(after), (mine(before), mine(after))
assert len(mine(after)) == 4
assert extra().startswith("Asked 3+2"), extra()
s.close()
print("\nknowledge smoke OK")
