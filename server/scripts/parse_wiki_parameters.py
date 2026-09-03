#!/usr/bin/env python3
"""
Mine structured-block parameter documentation (sub-keys) from the ho4i-wiki
docs and write them into assets/hoi4_data.json.

Structured triggers/effects like `add_timed_idea = { idea = X days = 180 }`
have a fixed set of sub-keys they accept. Those sub-keys are NOT generic
modifiers/triggers/effects — they are parameters of the owning block. The
wiki docs (hoi4-wiki/documentation/effects.md, triggers.md) document them
in each table row's params cell:

    |-id="add_timed_idea"
    |add_timed_idea
    |<code>idea = <idea></code><br>The idea to add.
    <code>days = <int> / <variable></code><br>The number of days ...

This script parses that cell (rows span multiple lines; continuation lines
without a leading `|` belong to the current cell) and fills the `parameters`
map of each matching entity in the JSON:

    "add_timed_idea": {
        "parameters": {
            "idea":   { "type": "string", "value_type": "idea",   "optional": false, "repeated": false, "description": "The idea to add." },
            "days":   { "type": "int / variable", "value_type": "int", ... }
        }
    }

Semantics:
- ADDITIVE + WHITELIST: only entities documented in the wiki get a
  non-empty `parameters` map; entities without docs are left untouched
  (never deleted, never cleared). Params present in the JSON but absent
  from the docs are dropped for the entities we DO parse.
- `value_type` = the first `<tag>` in the spec (e.g. <idea>, <country>,
  <equipment>) — the cross-reference kind consumers can map onto scanner
  entities (goto-definition / value completion). Primitive tags (int,
  string, bool, ...) are kept verbatim; consumers ignore unmapped kinds.
- `optional` / `repeated` are best-effort heuristics on the description
  wording ("optional", "multiple") — display hints only, never validation.
- Curation knobs at the bottom: EXCLUDE_ENTITIES (skip a badly-parsed
  entity entirely) and PARAM_OVERRIDES (hand-patch individual params).

Usage:
    python3 server/scripts/parse_wiki_parameters.py          # write JSON
    DRY_RUN=1 python3 server/scripts/parse_wiki_parameters.py  # stats only
"""

import json
import os
import re
import sys

DRY_RUN = bool(os.environ.get("DRY_RUN"))

REPO_ROOT = os.path.dirname(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
)
V2_JSON = os.path.join(REPO_ROOT, "server", "assets", "hoi4_data.json")
EFFECTS_DOC = os.path.join(REPO_ROOT, "hoi4-wiki", "documentation", "effects.md")
TRIGGERS_DOC = os.path.join(REPO_ROOT, "hoi4-wiki", "documentation", "triggers.md")

# ── Curation knobs ──────────────────────────────────────────────────────────
# Entities whose wiki params cell parses into garbage. Remove an entry once
# the wiki row is fixed or an override below covers it.
#
# The entries below are DYNAMIC-KEY blocks: their inner keys are entity
# references (`has_equipment = { infantry_equipment_1 > 10 }`), not fixed
# sub-keys. Documenting them as `parameters` would wrongly restrict
# completions inside them, so they stay undocumented (completion falls back
# to the full trigger/effect list) until slice-4 reference-typed keys.
EXCLUDE_ENTITIES = {
    "set_popularities",  # <ideology> = <int>
    "add_unit_bonus",  # <subunit> = { ... }
    "add_state_modifier",  # <string> = { ... } (modifier bag)
    "strategic_province_location",  # <string> = <int>
    "strategic_state_location",  # <string> = <int>
    "construct_building_in_random_province",  # <building> = <int>
    "ideology_support_trigger",  # <ideology> = <int>
    "compare_autonomy_state",  # <country> = <int>
    "has_equipment",  # <equipment> = <int> / <variable>
    "owns_or_subject_of",  # value-only + odd row shape
}

# Hand-written parameter patches applied AFTER doc parsing. Use for params
# the wiki formats awkwardly (block params, "X scope" prose, no `key =` prefix).
PARAM_OVERRIDES = {
    "create_country_leader": {
        "traits": {
            "type": "block",
            "value_type": "trait",
            "description": "The trait to add. Can add multiple.",
            "optional": False,
            "repeated": True,
        }
    },
    # `division_template` is BOTH an effect and the top-level definition block
    # used in history/units/*.txt and OOB files. The wiki documents name /
    # regiments / support, but the row parser can't reach them: `name` has no
    # `<type>` tag and regiments/support live in a <pre> example rather than a
    # <code> group. Without these three, a vanilla definition block highlights
    # inconsistently — `division_names_group` and `priority` resolve as block
    # parameters while `name` and `regiments` (present in all 804 vanilla
    # definition blocks) do not. All three are valid in both shapes.
    "division_template": {
        "name": {
            "type": "string",
            "value_type": "string",
            "description": "The name of the division.",
            "optional": False,
            "repeated": False,
        },
        "regiments": {
            "type": "block",
            "value_type": "",
            "description": (
                "The composition of the division. Sub-units are defined in "
                "common/units/*.txt files."
            ),
            "optional": False,
            "repeated": False,
        },
        "support": {
            "type": "block",
            "value_type": "",
            "description": (
                "The support companies of the division. Sub-units are defined "
                "in common/units/*.txt files."
            ),
            "optional": True,
            "repeated": False,
        },
    },
}

# ── Wiki table parsing ──────────────────────────────────────────────────────

ROW_ID_RE = re.compile(r'\|-id="([^"]+)"')


def parse_rows(filepath):
    """Split a wiki table into (entity_id, name, cells) rows.

    Row layout: `|-id="X"` starts a row; the next `|`-prefixed line is the
    name cell; every following `|`-prefixed line starts a new cell, and
    lines WITHOUT a leading `|` continue the current cell (the params cell
    spans several such lines).
    """
    with open(filepath, "r", encoding="utf-8") as f:
        lines = f.read().split("\n")
    rows = []
    i, n = 0, len(lines)
    while i < n:
        m = ROW_ID_RE.match(lines[i])
        if not m:
            i += 1
            continue
        eid = m.group(1)
        i += 1
        name = None
        if i < n and lines[i].startswith("|") and not lines[i].startswith("|-"):
            name = lines[i].lstrip("|").strip()
            i += 1
        cells, cur = [], None
        while i < n and not ROW_ID_RE.match(lines[i]):
            line = lines[i]
            if line.startswith("|"):
                if cur is not None:
                    cells.append(cur)
                cur = line[1:]
            else:
                cur = (cur or "") + "\n" + line
            i += 1
        if cur is not None:
            cells.append(cur)
        rows.append((eid, name, cells))
    return rows


# ── Parameter extraction ────────────────────────────────────────────────────

CODE_RE = re.compile(r"<code>(.*?)</code>")
KEY_TYPE_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?)\s*$")
TYPE_TAG_RE = re.compile(r"<([A-Za-z_][A-Za-z0-9_./]*)>")

WIKI_MARKUP_RE = [
    (re.compile(r"\{\{path\|([^}]+)\}\}"), r"\1"),
    (re.compile(r"\{\{Main\|([^}]+)\}\}"), r"\1"),
    (re.compile(r"\{\{[^}]+\}\}"), ""),
    (re.compile(r"\[\[([^|\]]+)\|([^\]]+)\]\]"), r"\2"),
    (re.compile(r"\[\[([^\]]+)\]\]"), r"\1"),
    (re.compile(r"'''"), ""),
    (re.compile(r"`"), ""),
    (re.compile(r"<br\s*/?>"), " "),
    (re.compile(r"<[^>]+>"), " "),
]


def clean_wiki_text(text):
    """Strip wiki markup from a description fragment."""
    out = text
    for pat, repl in WIKI_MARKUP_RE:
        out = pat.sub(repl, out)
    out = out.replace("\n", " ")
    return re.sub(r"\s+", " ", out).strip()


def parse_param_spec(spec):
    """Split `<code>idea = <idea></code>` into (key, type, value_type).

    Returns None when the code group is not a `key = ...` assignment
    (e.g. prose mentions, `{ ... }` block examples).
    """
    km = KEY_TYPE_RE.match(spec)
    if not km:
        return None
    key = km.group(1).lower()
    rest = km.group(2).strip()
    if rest.startswith("{"):
        return (key, "block", "")
    tags = TYPE_TAG_RE.findall(rest)
    if not tags:
        # No <type> tags: fall back to the raw value text, stripped of any
        # angle brackets (handles specs like `key = <mod ID>`).
        clean = re.sub(r"[<>]", " ", rest).strip()
        if not clean:
            return (key, "", "")
        return (key, clean[:32], "")
    type_str = " / ".join(t.lower() for t in tags)
    return (key, type_str, tags[0].lower())


def extract_params(cells):
    """First cell containing `key = <type>` code groups -> parameter dict.

    Returns (params, drift) where drift flags `key = ...` code groups that
    failed to parse (a real format surprise worth a warning). Bare value
    tags (`<code><flag></code>`, used by value-only effects like
    `clr_global_flag = X`) and prose `<code>` mentions are not drift.
    """
    params = {}
    drift = False
    for cell in cells:
        codes = CODE_RE.findall(cell)
        if not codes:
            continue
        parsed_any = False
        for m in CODE_RE.finditer(cell):
            code = m.group(1)
            # Value-only form (`<code><flag></code>`) — no key, not drift.
            if "=" not in code:
                continue
            parsed = parse_param_spec(code)
            if parsed is None:
                drift = True  # `key = ...` code that failed to parse
                continue
            parsed_any = True
            key, type_str, value_type = parsed
            # Description = text between this group's closing tag and the
            # next `<code>` (or end of cell).
            desc_end = cell.find("<code>", m.end())
            desc = cell[m.end() : desc_end if desc_end != -1 else len(cell)]
            desc = clean_wiki_text(desc)
            optional = bool(re.search(r"\boptional\b", desc, re.IGNORECASE))
            repeated = bool(
                re.search(
                    r"\bmultiple\b|\brepeatedly\b|\bcan add more than one\b",
                    desc,
                    re.IGNORECASE,
                )
            )
            params[key] = {
                "type": type_str,
                "value_type": value_type,
                "description": desc,
                "optional": optional,
                "repeated": repeated,
            }
        if parsed_any:
            return params, drift
    return params, drift


def is_valid_param(entity_id, key, pdef):
    """Reject/repair parameters the wiki parser produced from a malformed row.

    Returns (keep, note). When `keep` is True the param may still have been
    repaired in place (note explains what).

    Failure shapes seen in the wiki tables:

    1. Self-referential: value-only entities (`is_puppet = yes`,
       `activate_shine_on_focus = <focus>`) document their own name as the
       sub-key, which would suggest `is_puppet = { is_puppet = ... }`.
       These are DROPPED — the block takes a bare value, not a sub-key.
    2. Noise type: the `<type>` cell held only punctuation (`'\"\"'` for
       `create_unit.division`, `'???'` for `..._temp.scorer`). The PARAM is
       real — only its type is unknown — so the type/value_type are cleared
       and the param is KEPT. A numeric type like `0-1` or `365` is a
       legitimate wiki value-range hint and is left untouched.
    3. Degenerate description: punctuation-only leftovers like `': :'` from a
       split table cell. Cleared, param kept.

    Consumers treat `parameters` as authoritative for highlighting and
    completion, so a bad *value* is worse than a missing one — but dropping a
    real sub-key would silently narrow completion, which is worse still.
    """
    if key == entity_id:
        return False, "self-referential"
    if not re.fullmatch(r"[a-z_][a-z0-9_]*", key):
        return False, "non-identifier key"

    notes = []
    ptype = (pdef.get("type") or "").strip()
    # Noise-only type (no alphanumerics at all, or nothing but '?').
    if ptype and (not re.search(r"[A-Za-z0-9]", ptype) or re.fullmatch(r"[?]+", ptype)):
        pdef["type"] = ""
        pdef["value_type"] = ""
        notes.append(f"cleared noise type {ptype!r}")
    desc = (pdef.get("description") or "").strip()
    if desc and not re.search(r"[A-Za-z0-9]", desc):
        pdef["description"] = ""
        notes.append(f"cleared noise description {desc!r}")
    return True, "; ".join(notes)


def apply_overrides(params, entity_id):
    for key, patch in PARAM_OVERRIDES.get(entity_id, {}).items():
        merged = params.get(key, {})
        merged.update(patch)
        params[key] = merged


# ── Main ────────────────────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("Parsing wiki effect/trigger parameter docs...")
    print("=" * 60)
    if DRY_RUN:
        print("MODE: DRY_RUN (stats only, no writes)")

    if not os.path.exists(V2_JSON):
        print(f"ERROR: {V2_JSON} not found", file=sys.stderr)
        sys.exit(1)

    with open(V2_JSON, "r", encoding="utf-8") as f:
        v2 = json.load(f)

    # entity_id -> params, across both docs
    doc_params = {}
    format_drift = []
    for doc, label in ((EFFECTS_DOC, "effects"), (TRIGGERS_DOC, "triggers")):
        if not os.path.exists(doc):
            print(f"WARNING: {doc} not found, skipping", file=sys.stderr)
            continue
        rows = parse_rows(doc)
        count = 0
        for eid, name, cells in rows:
            if not cells:
                continue
            params, saw_codes = extract_params(cells)
            if params:
                doc_params[eid] = params
                count += 1
            elif saw_codes:
                format_drift.append(eid)
        print(f"{label}: {count}/{len(rows)} rows yielded parameters")

    print(f"\nTotal entities with documented parameters: {len(doc_params)}")
    drift_unhandled = [e for e in format_drift if e not in EXCLUDE_ENTITIES]
    if drift_unhandled:
        print(
            f"WARNING: {len(drift_unhandled)} rows had <code> tags but no "
            f"parseable params (possible format drift): {drift_unhandled[:20]}"
        )

    # ── Merge into JSON (additive whitelist) ───────────────────────────────
    stats = {"updated": 0, "excluded": 0, "params": 0, "unparsed_same": 0, "rejected": 0, "repaired": 0, "cleared": 0}
    rejected_detail = []
    for family in ("triggers", "effects", "modifiers"):
        for entity_id, entity in v2[family].items():
            # IDs in the JSON are the row id verbatim (may differ in case /
            # parens from doc names). Match case-insensitively.
            match = None
            for doc_id in doc_params:
                if doc_id.lower() == entity_id.lower() or doc_id.lower() == entity_id.strip("()").lower():
                    match = doc_id
                    break
            if match is None:
                continue
            if entity_id in EXCLUDE_ENTITIES:
                stats["excluded"] += 1
                continue
            params = dict(doc_params[match])
            apply_overrides(params, entity_id)
            # Drop/repair malformed rows AFTER overrides, so a hand-written
            # patch can rescue a param the wiki parsed badly.
            for key in list(params):
                keep, note = is_valid_param(entity_id, key, params[key])
                if not keep:
                    del params[key]
                    stats["rejected"] += 1
                    rejected_detail.append(f"{family}:{entity_id}.{key} — DROPPED ({note})")
                elif note:
                    stats["repaired"] += 1
                    rejected_detail.append(f"{family}:{entity_id}.{key} — repaired ({note})")
            if params:
                entity["parameters"] = params
                stats["updated"] += 1
                stats["params"] += len(params)
            else:
                # Every documented param was rejected (e.g. a value-only
                # entity whose only "param" was self-referential). Clear any
                # map left over from a previous run — otherwise re-running the
                # script can never remove a bad entry it wrote earlier.
                if entity.pop("parameters", None):
                    stats["cleared"] += 1

    print(f"\nEntities updated: {stats['updated']}")
    print(f"Excluded (curation): {stats['excluded']}")
    print(f"Total parameters written: {stats['params']}")
    if rejected_detail:
        print(f"Malformed params — dropped: {stats['rejected']}, repaired: {stats['repaired']}")
        for r in rejected_detail:
            print(f"  - {r}")

    # Sanity spot-checks
    for probe in ("add_timed_idea", "swap_ideas", "add_opinion_modifier"):
        for family in ("effects", "triggers"):
            ent = v2[family].get(probe)
            if ent and ent.get("parameters"):
                keys = sorted(ent["parameters"].keys())
                print(f"  {probe}: {keys}")
                break

    if DRY_RUN:
        print("\nDRY RUN — no changes written")
        return

    v2["version"] = 3  # schema grew a live `parameters` map
    with open(V2_JSON, "w", encoding="utf-8") as f:
        json.dump(v2, f, indent=2, ensure_ascii=False)
    print(f"\nWrote {V2_JSON} (version -> {v2['version']})")


if __name__ == "__main__":
    main()
