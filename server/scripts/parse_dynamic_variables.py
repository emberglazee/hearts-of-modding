#!/usr/bin/env python3
"""
Parse dynamic_variables_documentation.md into hoi4_data_v2.json.

Reads the official Paradox docs at:
  ~/.steam/steam/steamapps/common/Hearts of Iron IV/documentation/dynamic_variables_documentation.md

and merges a new top-level key `dynamic_variables` into server/assets/hoi4_data_v2.json.

Each entry:
  "faction_members": {
    "name": "faction_members",
    "description": "array of faction members",
    "scopes": {"usage": ["Country"], "usage_restriction": ""},
    "is_array": true
  }

Scopes are mapped from the `## Dynamic variables for scope X` headings.
`is_array` is true when the description contains "array" (case-insensitive);
otherwise the entry is a scalar dynamic variable.

The script is idempotent and preserves tab indentation (\\t) to match the
hand-maintained hoi4_data_v2.json style. Run with DRY_RUN=1 to preview.

Usage:
  python3 server/scripts/parse_dynamic_variables.py
  DRY_RUN=1 python3 server/scripts/parse_dynamic_variables.py
"""
import json
import os
import re
import sys

DRY_RUN = bool(os.environ.get("DRY_RUN"))

DOC_PATH = os.path.expanduser(
    "~/.steam/steam/steamapps/common/Hearts of Iron IV/documentation/dynamic_variables_documentation.md"
)
V2_JSON = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "assets/hoi4_data_v2.json",
)

# Doc scope heading -> LSP Scope enum value
SCOPE_MAP = {
    "country": "Country",
    "global": "Global",
    "state": "State",
    "unit_leader": "Character",
    # No dedicated LSP scope for these; treat as Global so they remain
    # queryable but don't spuriously narrow. If we ever add dedicated
    # scopes, update the map and bump the data version.
    "military_industrial_organization": "Global",
    "special_project": "Global",
}

SCOPE_HEADING_RE = re.compile(r"^## Dynamic variables for scope (\w+)", re.MULTILINE)
ENTRY_RE = re.compile(r"^### (\w+)\s*\n\* description: ([^\n]+)", re.MULTILINE)


def parse_docs(path):
    if not os.path.exists(path):
        print(f"ERROR: {path} not found", file=sys.stderr)
        sys.exit(1)
    text = pathlib_read(path)
    # Find scope headings with positions
    scopes = [(m.start(), m.group(1)) for m in SCOPE_HEADING_RE.finditer(text)]
    if not scopes:
        print("ERROR: no scope headings found", file=sys.stderr)
        sys.exit(1)
    dynamic_vars = {}
    for m in ENTRY_RE.finditer(text):
        name, desc = m.group(1).strip(), m.group(2).strip()
        pos = m.start()
        scope_raw = None
        for s_pos, s_name in reversed(scopes):
            if s_pos < pos:
                scope_raw = s_name
                break
        if scope_raw is None:
            print(f"WARNING: no scope for {name}, skipping", file=sys.stderr)
            continue
        lsp_scope = SCOPE_MAP.get(scope_raw.lower())
        if lsp_scope is None:
            print(f"WARNING: no LSP mapping for doc scope '{scope_raw}' (var {name}), using Global", file=sys.stderr)
            lsp_scope = "Global"
        is_array = "array" in desc.lower()
        # Some array-like dynamic variables are phrased as "all X" in the
        # docs without the word "array" (e.g. army_leaders). Keep them as
        # arrays so `array = army_leaders` doesn't spuriously warn.
        if name.lower() in ("army_leaders", "navy_leaders", "operatives"):
            is_array = True
        key = name  # preserve case as in docs (all lower anyway)
        if key in dynamic_vars:
            # Merge scopes for names that appear in multiple doc sections
            # (e.g. `num_battalions` in both country and unit_leader).
            existing = dynamic_vars[key]
            usage = existing["scopes"]["usage"]
            if lsp_scope not in usage:
                usage.append(lsp_scope)
                usage.sort()
            # is_array should be consistent; if either is true, keep true
            if is_array:
                existing["is_array"] = True
            continue
        dynamic_vars[key] = {
            "name": name,
            "description": desc,
            "scopes": {"usage": [lsp_scope], "usage_restriction": ""},
            "is_array": is_array,
        }
    return dynamic_vars


def pathlib_read(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def main():
    print("=" * 60)
    print("Parsing dynamic_variables_documentation.md")
    print("=" * 60)
    print(f"Doc: {DOC_PATH}")
    print(f"JSON: {V2_JSON}")

    dynamic_vars = parse_docs(DOC_PATH)
    arrays = sum(1 for v in dynamic_vars.values() if v["is_array"])
    scalars = len(dynamic_vars) - arrays
    print(f"\nFound {len(dynamic_vars)} dynamic variables ({arrays} arrays, {scalars} scalars)")
    # Show sample
    for k in sorted(dynamic_vars.keys())[:5]:
        v = dynamic_vars[k]
        print(f"  {k}: is_array={v['is_array']} scope={v['scopes']['usage']} desc={v['description'][:60]}")

    # Load existing V2 JSON
    if not os.path.exists(V2_JSON):
        print(f"ERROR: {V2_JSON} not found", file=sys.stderr)
        sys.exit(1)
    with open(V2_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    old_count = len(data.get("dynamic_variables", {}))
    data["dynamic_variables"] = dict(sorted(dynamic_vars.items()))
    # Bump version if new key added
    if "dynamic_variables" not in data or data.get("version", 0) < 4:
        # Only bump if we are adding the key for first time or version < 4
        # Keep idempotent: if already 4, don't keep bumping
        if data.get("version", 0) < 4:
            data["version"] = 4
            print(f"\nBumped version to 4 (new dynamic_variables key)")

    print(f"\nPreviously {old_count} dynamic_variables, now {len(dynamic_vars)}")
    if DRY_RUN:
        print("[DRY_RUN] not writing")
        return

    # Write with tab indent to match hand-maintained style (see earlier manual edits)
    with open(V2_JSON, "w", encoding="utf-8") as f:
        json.dump(data, f, indent="\t", ensure_ascii=False)
        f.write("\n")
    print(f"Wrote {V2_JSON}")


if __name__ == "__main__":
    main()
