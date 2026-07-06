#!/usr/bin/env python3
"""
Parse Paradox's official triggers_documentation.md, effects_documentation.md,
and modifiers_documentation.md to rewrite scope registrations in hoi4_data_v2.json.

WHITELIST APPROACH: if an entity isn't in the official docs, DELETE it.
"""

import json
import os
import re
import sys

# Paths
DOC_DIR = os.path.expanduser(
    "~/.steam/steam/steamapps/common/Hearts of Iron IV/documentation"
)
TRIGGERS_DOC = os.path.join(DOC_DIR, "triggers_documentation.md")
EFFECTS_DOC = os.path.join(DOC_DIR, "effects_documentation.md")
MODIFIERS_DOC = os.path.join(DOC_DIR, "modifiers_documentation.md")
V2_JSON = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "assets/hoi4_data_v2.json",
)


# ── Scope mapping from doc scope names → LSP Scope enum ──────────────────────

TRIGGER_EFFECT_SCOPE_MAP = {
    "ACE": "Ace",
    "CHARACTER": "Character",
    "COMBATANT": "Unit",
    "COUNTRY": "Country",
    "FACTION": "Country",
    "INDUSTRIAL_ORG": "Country",
    "OPERATION": "Country",
    "PURCHASE_CONTRACT": "Country",
    "RAID_INSTANCE": "Country",
    "SPECIAL_PROJECT": "Country",
    "STATE": "State",
    "STRATEGIC_REGION": "StrategicRegion",
    "any": "Global",
}

# Modifier docs use functional categories. Map them to the best LSP scope.
MODIFIER_SCOPE_MAP = {
    "aggressive": "Country",
    "ai": "Country",
    "air": "Country",
    "army": "Country",
    "autonomy": "Country",
    "character": "Character",
    "country": "Country",
    "defensive": "Country",
    "faction": "Country",
    "government_in_exile": "Country",
    "intelligence_agency": "Country",
    "military_advancements": "Country",
    "naval": "Country",
    "peace": "Country",
    "politics": "Country",
    "scientist": "Character",
    "state": "State",
    "unit_leader": "Unit",
    "war_production": "Country",
}


def parse_doc_sections(filepath, prefix):
    """
    Parse a documentation .md file and extract entity→scopes mapping.

    Looks for sections matching:
        ## <prefix> for scope <SCOPE_NAME>
    then collects every bullet `* [entity_name](#entity_name)` under that section.

    Returns: {entity_name_lower: set(scope_enum_values)}
    """
    if not os.path.exists(filepath):
        print(f"WARNING: {filepath} not found, skipping", file=sys.stderr)
        return {}

    with open(filepath, "r", encoding="utf-8") as f:
        text = f.read()

    lines = text.split("\n")

    entity_scopes = {}

    current_scope_upper = None
    in_scope_section = False

    section_pattern = re.compile(
        r"^##\s+" + re.escape(prefix) + r"\s+for\s+scope\s+(\S+)",
        re.IGNORECASE,
    )
    bullet_pattern = re.compile(r"^\*\s+\[([^\]]+)\]")

    for line in lines:
        # Check for section header
        m = section_pattern.match(line)
        if m:
            current_scope_upper = m.group(1)  # Keep original case
            in_scope_section = True
            continue

        # If we hit another ## section, stop collecting for current scope
        if in_scope_section and line.startswith("## ") and not section_pattern.match(line):
            in_scope_section = False
            current_scope_upper = None

        # Collect bullet points under the current scope section
        if in_scope_section and current_scope_upper is not None:
            m2 = bullet_pattern.match(line)
            if m2:
                entity_name = m2.group(1).strip().lower()
                if entity_name not in entity_scopes:
                    entity_scopes[entity_name] = set()
                # Preserve case for matching against the map
                entity_scopes[entity_name].add(current_scope_upper)

    return entity_scopes


def parse_triggers_doc():
    """Parse triggers_documentation.md"""
    return parse_doc_sections(TRIGGERS_DOC, "Triggers")


def parse_effects_doc():
    """Parse effects_documentation.md"""
    return parse_doc_sections(EFFECTS_DOC, "Effects")


def parse_modifiers_doc():
    """Parse modifiers_documentation.md"""
    return parse_doc_sections(MODIFIERS_DOC, "Modifiers")


def map_doc_scopes_to_lsp(doc_scopes_set, scope_map):
    """Convert a set of doc scope names to a sorted list of LSP scope enum values."""
    result = set()
    for s in doc_scopes_set:
        mapped = scope_map.get(s)
        if mapped:
            result.add(mapped)
        else:
            print(f"  WARNING: No scope mapping for '{s}', skipping", file=sys.stderr)
    return sorted(result)


def main():
    print("=" * 60)
    print("Parsing official Paradox documentation...")
    print("=" * 60)

    # ── Parse docs ────────────────────────────────────────────────────────────
    trigger_scopes = parse_triggers_doc()
    effect_scopes = parse_effects_doc()
    modifier_scopes = parse_modifiers_doc()

    print(f"\nTriggers in docs: {len(trigger_scopes)}")
    print(f"Effects in docs:  {len(effect_scopes)}")
    print(f"Modifiers in docs: {len(modifier_scopes)}")

    # For debugging: print first few
    if trigger_scopes:
        sample = list(sorted(trigger_scopes.keys()))[:5]
        print(f"  Sample trigger entities: {sample}")
        for e in sample:
            print(f"    {e}: {trigger_scopes[e]}")

    # ── Read V2 JSON ──────────────────────────────────────────────────────────
    print(f"\nReading {V2_JSON}...")
    with open(V2_JSON, "r", encoding="utf-8") as f:
        v2_data = json.load(f)

    version = v2_data.get("version")
    print(f"Version: {version}")
    print(f"Original triggers: {len(v2_data.get('triggers', {}))}")
    print(f"Original effects:  {len(v2_data.get('effects', {}))}")
    print(f"Original modifiers: {len(v2_data.get('modifiers', {}))}")

    # ── Process triggers ──────────────────────────────────────────────────────
    print("\n" + "=" * 60)
    print("Processing TRIGGERS...")
    print("=" * 60)

    v2_triggers = v2_data.get("triggers", {})
    new_triggers = {}
    deleted_count = 0
    updated_count = 0

    for entity_name in sorted(v2_triggers.keys()):
        # Normalize: V2 JSON uses parenthetical names like (building_count_trigger)
        # but docs use bare names like building_count_trigger
        lookup_name = entity_name.strip("()").lower()

        if lookup_name in trigger_scopes:
            # Keep entity, update scopes from docs
            doc_scopes = trigger_scopes[lookup_name]
            lsp_scopes = map_doc_scopes_to_lsp(doc_scopes, TRIGGER_EFFECT_SCOPE_MAP)

            entry = v2_triggers[entity_name]
            old_scopes = entry.get("scopes", {}).get("usage", [])
            entry["scopes"] = {"usage": lsp_scopes, "usage_restriction": ""}
            new_triggers[entity_name] = entry

            if set(old_scopes) != set(lsp_scopes):
                print(f"  UPDATED: {entity_name}: {old_scopes} → {lsp_scopes}")
                updated_count += 1
        else:
            print(f"  DELETED: {entity_name} (not in official docs)")
            deleted_count += 1

    v2_data["triggers"] = new_triggers
    print(f"\nTriggers: {len(new_triggers)} kept, {deleted_count} deleted, {updated_count} updated")

    # ── Process effects ────────────────────────────────────────────────────────
    print("\n" + "=" * 60)
    print("Processing EFFECTS...")
    print("=" * 60)

    v2_effects = v2_data.get("effects", {})
    new_effects = {}
    deleted_count = 0
    updated_count = 0

    for entity_name in sorted(v2_effects.keys()):
        lookup_name = entity_name.strip("()").lower()

        if lookup_name in effect_scopes:
            doc_scopes = effect_scopes[lookup_name]
            lsp_scopes = map_doc_scopes_to_lsp(doc_scopes, TRIGGER_EFFECT_SCOPE_MAP)

            entry = v2_effects[entity_name]
            old_scopes = entry.get("scopes", {}).get("usage", [])
            entry["scopes"] = {"usage": lsp_scopes, "usage_restriction": ""}
            new_effects[entity_name] = entry

            if set(old_scopes) != set(lsp_scopes):
                print(f"  UPDATED: {entity_name}: {old_scopes} → {lsp_scopes}")
                updated_count += 1
        else:
            print(f"  DELETED: {entity_name} (not in official docs)")
            deleted_count += 1

    v2_data["effects"] = new_effects
    print(f"\nEffects: {len(new_effects)} kept, {deleted_count} deleted, {updated_count} updated")

    # ── Process modifiers ─────────────────────────────────────────────────────
    print("\n" + "=" * 60)
    print("Processing MODIFIERS...")
    print("=" * 60)

    v2_modifiers = v2_data.get("modifiers", {})
    new_modifiers = {}
    deleted_count = 0
    updated_count = 0

    for entity_name in sorted(v2_modifiers.keys()):
        lookup_name = entity_name.strip("()").lower()

        if lookup_name in modifier_scopes:
            doc_scopes = modifier_scopes[lookup_name]
            lsp_scopes = map_doc_scopes_to_lsp(doc_scopes, MODIFIER_SCOPE_MAP)

            entry = v2_modifiers[entity_name]
            old_scopes = entry.get("scopes", {}).get("usage", [])
            entry["scopes"] = {"usage": lsp_scopes, "usage_restriction": ""}
            new_modifiers[entity_name] = entry

            if set(old_scopes) != set(lsp_scopes):
                print(f"  UPDATED: {entity_name}: {old_scopes} → {lsp_scopes}")
                updated_count += 1
        else:
            print(f"  DELETED: {entity_name} (not in official docs)")
            deleted_count += 1

    v2_data["modifiers"] = new_modifiers
    print(f"\nModifiers: {len(new_modifiers)} kept, {deleted_count} deleted, {updated_count} updated")

    # ── Write output ──────────────────────────────────────────────────────────
    print(f"\nWriting updated {V2_JSON}...")
    with open(V2_JSON, "w", encoding="utf-8") as f:
        json.dump(v2_data, f, indent=2, ensure_ascii=False)

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Remaining triggers:  {len(v2_data['triggers'])}")
    print(f"Remaining effects:   {len(v2_data['effects'])}")
    print(f"Remaining modifiers: {len(v2_data['modifiers'])}")
    print(f"Version preserved:   {v2_data['version']}")
    print("Done!")


if __name__ == "__main__":
    main()
