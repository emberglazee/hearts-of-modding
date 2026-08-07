#!/usr/bin/env python3
"""LSP smoke test for structured-block parameters: completions + hover.

Speaks JSON-RPC (Content-Length framed) to the hom-lsp binary and checks:
1. completion inside `add_timed_idea = { ... }` ranks its documented params
   (idea/days/months/years) FIRST while still returning the full generic list
   — the params are additive, never a replacement
2. completion in an undocumented context still returns the full list
3. hover on `days` shows the parameter section with its description
"""
import json
import os
import subprocess
import sys
import time

BIN = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "target", "debug", "hom-lsp",
)
URI = "file:///tmp/hoi4_smoke/events/param_test.txt"
TEXT = """\
add_timed_idea = {
    idea = SPE_new_southern_worker_rights
    days = 180
}
add_political_power = 100
"""

FAIL = []


def frame(obj):
    body = json.dumps(obj).encode()
    return b"Content-Length: %d\r\n\r\n%s" % (len(body), body)


def main():
    proc = subprocess.Popen(
        [BIN, "--stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )

    def send(obj):
        proc.stdin.write(frame(obj))
        proc.stdin.flush()

    def read_msg(timeout=30):
        """Read LSP messages until a RESPONSE (has `id`) arrives; skip
        server-initiated notifications (logMessage etc.)."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            buf = b""
            while time.time() < deadline:
                chunk = proc.stdout.read(1)
                if not chunk:
                    time.sleep(0.02)
                    continue
                buf += chunk
                if buf.endswith(b"\r\n\r\n"):
                    break
            if not buf.endswith(b"\r\n\r\n"):
                raise TimeoutError("no LSP message")
            headers = buf.decode()
            length = int(
                [l for l in headers.split("\r\n") if l.lower().startswith("content-length")][
                    0
                ].split(":")[1]
            )
            msg = json.loads(proc.stdout.read(length))
            # Responses have `id` and NO `method`; server→client REQUESTS
            # (client/registerCapability) have both — skip those.
            if "id" in msg and "method" not in msg:
                return msg
        raise TimeoutError("no LSP response")

    send(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": None,
                "rootUri": "file:///tmp/hoi4_smoke",
                "capabilities": {},
            },
        }
    )
    init = read_msg()
    print("initialize:", "ok" if init.get("result") else "FAIL", list(init.get("result", {}))[:3])

    send({"jsonrpc": "2.0", "method": "initialized", "params": {}})
    send(
        {
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {"uri": URI, "languageId": "paradox", "version": 1, "text": TEXT}
            },
        }
    )

    # ── Completion inside add_timed_idea (cursor on the `idea` line) ────────
    send(
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": URI},
                "position": {"line": 1, "character": 4},
            },
        }
    )
    comp = read_msg()
    result = comp.get("result", [])
    items = result if isinstance(result, list) else result.get("items", []) or []
    labels = [i.get("label") for i in items]
    params = ["days", "idea", "months", "years"]
    # Documented params must be present AND ranked first (they carry a "0_"
    # sort_text), but they must NOT replace the generic list — the wiki only
    # documents a block's scalar sub-keys, so suppressing everything else
    # would hide legitimate keys.
    missing = [p for p in params if p not in labels]
    ranked_first = sorted(labels[: len(params)]) == params
    has_fallback = len(items) > 100
    if not missing and ranked_first and has_fallback:
        print(
            f"completion(add_timed_idea): PASS -> params first {labels[:4]}, "
            f"{len(items)} items total (additive)"
        )
    else:
        why = []
        if missing:
            why.append(f"missing params {missing}")
        if not ranked_first:
            why.append(f"params not ranked first (got {labels[:6]})")
        if not has_fallback:
            why.append(f"generic list suppressed (only {len(items)} items)")
        FAIL.append(f"completion inside add_timed_idea: {'; '.join(why)}")
        print("completion(add_timed_idea): FAIL ->", "; ".join(why))

    # ── Completion at top level (undocumented context) → full list fallback ─
    send(
        {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": URI},
                "position": {"line": 4, "character": 4},
            },
        }
    )
    comp2 = read_msg()
    result2 = comp2.get("result", [])
    items2 = result2 if isinstance(result2, list) else result2.get("items", []) or []
    # At top level there is no documented enclosing block -> the full
    # scope-filtered list falls back. Country-scoped effects like
    # add_timed_idea / add_political_power are excluded at Global scope
    # (pre-existing filter); Global-scope effects like `if`/`add_to_array`
    # remain.
    labels2 = [i.get("label") for i in items2]
    if len(items2) > 100 and "if" in labels2 and "add_to_array" in labels2:
        print(f"completion(top-level fallback): PASS ({len(items2)} items, full list)")
    else:
        FAIL.append(f"top-level completion expected full list, got {len(items2)} items")

    # ── Hover on `days` (line 2, char 5) ────────────────────────────────────
    send(
        {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {"uri": URI},
                "position": {"line": 2, "character": 5},
            },
        }
    )
    hover = read_msg()
    contents = hover.get("result", {}).get("contents", {})
    value = contents.get("value", "") if isinstance(contents, dict) else ""
    if "Parameter" in value and "days" in value and "number of days" in value:
        print("hover(days): PASS ->", value.splitlines()[0])
    else:
        FAIL.append(f"hover on days expected parameter section, got: {value[:200]!r}")

    # ── Hover on a VALUE (idea ref) — should NOT be a param section ─────────
    send(
        {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {"uri": URI},
                "position": {"line": 1, "character": 14},
            },
        }
    )
    hover2 = read_msg()
    contents2 = hover2.get("result", {}).get("contents", {})
    value2 = contents2.get("value", "") if isinstance(contents2, dict) else ""
    if "Parameter" not in value2:
        print("hover(idea value): PASS (no param section for a value)")
    else:
        FAIL.append(f"hover on idea VALUE should not show a parameter section: {value2[:120]!r}")

    proc.terminate()
    print()
    if FAIL:
        print("FAILURES:")
        for f in FAIL:
            print("  -", f)
        sys.exit(1)
    print("ALL LSP SMOKE CHECKS PASSED")


if __name__ == "__main__":
    main()
