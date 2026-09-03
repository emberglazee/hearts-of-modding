#!/usr/bin/env python3
"""
V2 JSON Scope Rewriter

Uses empirical vanilla analysis data to rewrite scopes in hoi4_data.json.
Strategy:
  1. For each entity in V2 JSON, check vanilla analysis data
  2. If >90% usage in one scope → assign that scope confidently
  3. If moderate evidence (primary scope + secondary) → assign primary + checked secondary
  4. If no vanilla data or ambiguous → flag for subagent investigation
  5. Apply known corrections from empirical session findings
"""

import json
import os

VANILLA_ANALYSIS_PATH = "server/scripts/vanilla_scope_analysis.json"
V2_JSON_PATH = "server/assets/hoi4_data.json"
OUTPUT_PATH = "server/assets/hoi4_data.json"
FLAGGED_PATH = "server/scripts/flagged_ambiguous_entities.json"

# Scopes that are basically the same for effective_scope purposes
# NationalFocus → Country, FocusTree → Country
EFFECTIVE_SCOPE_MAP = {
    "NationalFocus": "Country",
    "FocusTree": "Country",
}

def load_data():
    with open(VANILLA_ANALYSIS_PATH) as f:
        vanilla = json.load(f)
    with open(V2_JSON_PATH) as f:
        v2 = json.load(f)
    return vanilla, v2

def get_entity_scopes(vanilla, entity_name):
    """Get scope data for an entity from vanilla analysis."""
    entities = vanilla.get("entities", {})
    return entities.get(entity_name)

def determine_scopes(vanilla_data):
    """
    Determine scopes for an entity based on vanilla analysis.
    Returns (scopes_list, confidence)
    confidence: 'high' (>90%), 'medium' (majority), 'low' (inconclusive), 'novanilla' (not found)
    """
    if vanilla_data is None:
        return None, "novanilla"
    
    total = vanilla_data["total"]
    scopes = vanilla_data["scopes"]
    
    # Filter out structural scopes that are really Country
    # NationalFocus and FocusTree scope usages are effectively Country
    effective_counts = {}
    for scope, count in scopes.items():
        effective = EFFECTIVE_SCOPE_MAP.get(scope, scope)
        effective_counts[effective] = effective_counts.get(effective, 0) + count
    
    # Sort by count descending
    sorted_scopes = sorted(effective_counts.items(), key=lambda x: -x[1])
    primary_scope = sorted_scopes[0][0]
    primary_count = sorted_scopes[0][1]
    
    # Calculate ratio of primary scope
    ratio = primary_count / total if total > 0 else 0
    
    # Get all scopes with significant usage (>5% of total or >10 counts)
    significant = [s for s, c in sorted_scopes if c / total > 0.05 or c >= 10]
    
    if ratio >= 0.90:
        return significant, "high"
    elif ratio >= 0.60:
        return significant, "medium"
    else:
        return significant, "low"


def main():
    vanilla, v2 = load_data()
    
    entity_categories = ["triggers", "effects", "modifiers"]
    flagged = []
    stats = {"high": 0, "medium": 0, "low": 0, "novanilla": 0, "skipped": 0}
    
    for category in entity_categories:
        entities = v2.get(category, {})
        for name, entity in entities.items():
            vanilla_data = get_entity_scopes(vanilla, name)
            scopes, confidence = determine_scopes(vanilla_data)
            
            if scopes is None and confidence == "novanilla":
                # Entity not found in vanilla analysis
                # Could be rarely used or not found by our parser
                # Flag for investigation, keep current scopes
                current_scopes = entity.get("scopes", {}).get("usage", [])
                if current_scopes:
                    flagged.append({
                        "category": category,
                        "name": name,
                        "reason": "not found in vanilla analysis, has current scopes",
                        "current_scopes": current_scopes,
                    })
                    stats["novanilla"] += 1
                else:
                    stats["skipped"] += 1
                continue
            
            # Clean scopes: remove ModifierBag (special scope not for entity registration)
            scopes = [s for s in scopes if s not in ("ModifierBag",)]
            
            if not scopes:
                stats["skipped"] += 1
                continue
            
            if confidence == "high":
                # High confidence — directly assign
                old_scopes = entity.get("scopes", {}).get("usage", [])
                if sorted(scopes) != sorted(old_scopes):
                    print(f"{category}/{name}: {old_scopes} -> {scopes} (high, {vanilla_data['total']} uses)")
                    entity["scopes"]["usage"] = scopes
                stats["high"] += 1
                
            elif confidence == "medium":
                # Medium confidence — check if scopes are reasonable
                old_scopes = entity.get("scopes", {}).get("usage", [])
                if sorted(scopes) != sorted(old_scopes):
                    print(f"{category}/{name}: {old_scopes} -> {scopes} (medium, {vanilla_data['total']} uses)")
                    entity["scopes"]["usage"] = scopes
                stats["medium"] += 1
                
            else:
                # Low confidence or ambiguous — flag for subagent
                if scopes:
                    flagged.append({
                        "category": category,
                        "name": name,
                        "reason": f"ambiguous scopes from {vanilla_data['total']} uses",
                        "vanilla_scopes": scopes,
                        "current_scopes": entity.get("scopes", {}).get("usage", []),
                    })
                stats["low"] += 1
    
    # Write updated V2 JSON
    with open(OUTPUT_PATH, 'w') as f:
        json.dump(v2, f, indent=2)
    
    # Write flagged entities
    with open(FLAGGED_PATH, 'w') as f:
        json.dump({
            "stats": stats,
            "flagged_count": len(flagged),
            "entities": flagged,
        }, f, indent=2)
    
    print(f"\n=== Results ===")
    print(f"High confidence (applied): {stats['high']}")
    print(f"Medium confidence (applied): {stats['medium']}")
    print(f"Low confidence (flagged): {stats['low']}")
    print(f"Not in vanilla (flagged if has scopes): {stats['novanilla']}")
    print(f"Skipped (no scopes): {stats['skipped']}")
    print(f"Total flagged for review: {len(flagged)}")
    print(f"Flagged written to: {FLAGGED_PATH}")


if __name__ == "__main__":
    main()
