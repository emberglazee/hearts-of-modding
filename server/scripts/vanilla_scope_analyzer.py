#!/usr/bin/env python3
"""
Vanilla HOI4 Scope Analyzer

Walks all vanilla HOI4 files, tracks scope context, and records
which triggers/effects/modifiers appear in which scopes.

Output: JSON with per-entity scope usage counts.

The scope model follows HOI4 engine behavior:
  Global   - Accessible from anywhere (variable ops, meta triggers)
  Country  - Country-level (stability, war, factories, politics)
  State    - State-level (buildings, resources, population)
  Character- Character/unit leader level (traits, skills, roles)
  Unit     - Division/ship/plane level (strength, org, equipment)
  Ace      - Ace pilot specific
  StrategicRegion - Strategic region specific
  Idea     - Idea/design company definitions

File-level initial scopes:
  common/abilities/         → Character
  common/aces/              → Ace
  common/decisions/         → Country
  common/national_focus/    → Country
  common/characters/        → Character
  common/countries/         → Country
  common/units/             → Unit
  common/unit_leader/       → Character
  common/ideas/             → Idea
  common/ideologies/        → Global
  common/defines/           → Global
  common/on_actions/        → Global
  common/scripted_effects/  → Global
  common/scripted_triggers/ → Global
  common/technology/        → Country
  common/technologies/      → Country
  common/buildings/         → State
  common/provinces/         → State
  common/strategic_regions/ → StrategicRegion
  events/                   → Global (event type determines scope)
  history/                  → Country
  map/                      → Global
  gfx/                      → Global
  music/                    → Global
  interface/                → Global
  common/operations/        → Country
  common/decisions/         → Country
  common/peace_conference/  → Country
  common/bookmarks/         → Country
  common/ai_strategy/       → Country
  common/ai_areas/          → Country
  common/bop/               → Country
  common/autonomous_states/ → Country
  common/characters/        → Character
  common/country_leader/    → Character
  common/ideas/             → Country (default for idea files)
  common/ideologies/        → Global
  common/intelligence_agencies/ → Country
  common/military_industrial_organization/ → Country

Transparent blocks (pass through current scope):
  AND, OR, NOT, if, limit, trigger, modifier, allowed, chance,
  ai_will_do, available, bypass, enable, on_start, immediate,
  option, after, completion_reward, custom_trigger_tooltip,
  custom_effect_tooltip, hidden_effect, hidden_trigger,
  effect_tooltip, random_list, random, for_loop_effect,
  while_loop_effect, for_each_scope_loop, for_each_loop,
  count_triggers, custom_override_tooltip

Chain targets (scope transitions):
  FROM country/state level -> depends on context
  ROOT -> first non-transparent scope
  OWNER -> Country (from Character)
  controller -> Country (from State)
  owner -> Country (from State)
  capital -> State (from Country)
"""

import os
import re
import json
import glob
from collections import defaultdict
from pathlib import Path

# ─── Scope Definitions ───────────────────────────────────────────────────────

SCOPES = {
    "Global", "Country", "State", "Character", "Unit", "Ace",
    "StrategicRegion", "Idea", "FocusTree", "NationalFocus",
    "ModifierBag", "MusicStation", "MusicTrack",
}

# File path → initial scope mapping
FILE_INITIAL_SCOPE = {
    "common/abilities": "Character",
    "common/aces": "Ace",
    "common/decisions": "Country",
    "common/national_focus": "Country",
    "common/characters": "Character",
    "common/countries": "Country",
    "common/unit_leader": "Character",
    "common/units": "Unit",
    "common/ideas": "Idea",
    "common/ideologies": "Global",
    "common/defines": "Global",
    "common/on_actions": "Global",
    "common/scripted_effects": "Global",
    "common/scripted_triggers": "Global",
    "common/technology": "Country",
    "common/technologies": "Country",
    "common/buildings": "State",
    "common/provinces": "State",
    "common/strategic_regions": "StrategicRegion",
    "common/operations": "Country",
    "common/peace_conference": "Country",
    "common/bookmarks": "Country",
    "common/ai_strategy": "Country",
    "common/ai_areas": "Country",
    "common/bop": "Country",
    "common/autonomous_states": "Country",
    "common/country_leader": "Character",
    "common/intelligence_agencies": "Country",
    "common/military_industrial_organization": "Country",
    "common/decisions": "Country",
    "history": "Country",
    "events": "Global",
}

# Keywords that are transparent (pass through scope)
TRANSPARENT_KEYWORDS = {
    "AND", "OR", "NOT", "if", "else", "else_if", "limit", "trigger",
    "modifier", "allowed", "chance", "ai_will_do", "available",
    "bypass", "enable", "on_start", "immediate", "option", "after",
    "completion_reward", "custom_trigger_tooltip", "custom_effect_tooltip",
    "custom_override_tooltip", "hidden_effect", "hidden_trigger",
    "effect_tooltip", "random_list", "random", "for_loop_effect",
    "while_loop_effect", "for_each_scope_loop", "for_each_loop",
    "count_triggers", "complete_tooltip", "available_if_capitulated",
    "bypass_if_unavailable", "allow_branch", "will_lead_to_war_with",
    "historical_ai", "joint_trigger", "supports_ai_strategy",
    "cancel_if_invalid", "continue_if_invalid", "daily_cost",
    "bypass_effect", "cancel_effect",
}

# Known scope keywords (these keys push their scope when they have a block value)
# Map from keyword → scope
SCOPE_PUSH_KEYWORDS = {
    # Country-scoped iterators/containers
    "country": "Country", "any_country": "Country", "every_country": "Country",
    "random_country": "Country", "all_country": "Country",
    "any_neighbor_country": "Country", "any_allied_country": "Country",
    "any_enemy_country": "Country", "any_other_country": "Country",
    "any_subject_country": "Country", "any_guaranteed_country": "Country",
    "any_occupied_country": "Country", "any_controlled_country": "Country",
    "any_country_with_core": "Country", "any_country_of": "Country",
    "any_country_with_original_tag": "Country", "any_home_area_neighbor_country": "Country",
    "all_owned_country": "Country", "all_core_country": "Country",
    "all_controlled_country": "Country",
    # These structural blocks are Country-scoped in practice
    "completion_reward": "Country", "completion_reward_joint_originator": "Country",
    "completion_reward_joint_member": "Country", "select_effect": "Country",
    # State-scoped iterators
    "state": "State", "any_state": "State", "every_state": "State",
    "random_state": "State", "all_state": "State",
    "any_neighbor_state": "State", "any_owned_state": "State",
    "any_controlled_state": "State", "any_core_state": "State",
    "any_state_in": "State", "any_state_of": "State",
    "any_home_state": "State",
    # Unit-scoped iterators
    "unit": "Unit", "any_unit": "Unit", "every_unit": "Unit",
    "random_unit": "Unit", "all_unit": "Unit",
    "any_division": "Unit", "every_division": "Unit",
    # Character-scoped iterators
    "character": "Character", "any_character": "Character",
    "every_character": "Character", "random_character": "Character",
    "all_character": "Character",
    # Unit Leader (maps to Character)
    "any_unit_leader": "Character", "every_unit_leader": "Character",
    "random_unit_leader": "Character", "all_unit_leader": "Character",
    "any_army_leader": "Character", "every_army_leader": "Character",
    "random_army_leader": "Character", "all_army_leader": "Character",
    "any_navy_leader": "Character", "every_navy_leader": "Character",
    "random_navy_leader": "Character", "all_navy_leader": "Character",
    # Operative
    "any_operative_leader": "Character", "every_operative_leader": "Character",
    "random_operative_leader": "Character", "all_operative_leader": "Character",
    "any_operative": "Character", "every_operative": "Character",
    "random_operative": "Character",
    # Scientist
    "any_scientist": "Character", "every_scientist": "Character",
    "random_scientist": "Character", "all_scientists": "Character",
    "any_active_scientist": "Character", "every_active_scientist": "Character",
    "random_active_scientist": "Character", "all_active_scientist": "Character",
    # Military Industrial Organization
    "any_military_industrial_organization": "Country",
    "every_military_industrial_organization": "Country",
    "random_military_industrial_organization": "Country",
    "all_military_industrial_organization": "Country",
    # MIO scope
    "mio": "Country",  # MIO definitions are at Country scope
    # Purchase contract
    "any_purchase_contract": "Country", "every_purchase_contract": "Country",
    "random_purchase_contract": "Country", "all_purchase_contract": "Country",
    # Strategic region
    "strategic_region": "StrategicRegion",
    # Focus tree
    "focus_tree": "FocusTree", "focus": "NationalFocus",
    "shared_focus": "NationalFocus", "joint_focus": "NationalFocus",
    # Building
    "building": "State", "building_group": "State",
    # Ideas
    "ideas": "Idea", "hidden_ideas": "Idea",
}

# Chain target scopes: key → target scope
# These are used inside blocks to reference another scope
CHAIN_TARGETS = {
    "FROM": None,  # context-dependent
    "ROOT": None,  # first non-transparent
    "OWNER": "Country",  # from Character → Country
    "controller": "Country",  # from State → Country
    "owner": "Country",  # from State/Unit → Country
    "capital": "State",  # from Country → State
    "tag": "Country",  # explicit country tag
}

# Module (subdirectory) scope overrides for specific event/script types
# These keywords inside blocks push the specified scope
EVENT_TYPE_SCOPES = {
    "country_event": "Country",
    "state_event": "State",
    "unit_leader_event": "Character",
    "operative_leader_event": "Character",
    "news_event": "Country",
}


def get_initial_scope(filepath):
    """Determine the initial scope for a file based on its path."""
    path = filepath.replace("\\", "/")
    for prefix, scope in sorted(FILE_INITIAL_SCOPE.items(), key=lambda x: -len(x[0])):
        if f"/{prefix}/" in path or path.startswith(f"{prefix}/"):
            return scope
    return "Global"


def parse_script_lines(lines):
    """
    Simple line-based HOI4 script parser.
    Returns a list of (key, value_type, value_text, line_number) tuples.
    Doesn't handle complex nesting but is good enough for scope tracking.
    """
    entries = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        # Skip comments and empty lines
        if not stripped or stripped.startswith('#'):
            i += 1
            continue
        
        # Try to match assignment: key = value
        # Match simple assignments first: key = value (on same line)
        m = re.match(r'^\s*([\w@._-]+)\s*=\s*(.+?)\s*$', line)
        if m:
            key = m.group(1)
            value_part = m.group(2).strip()
            
            if value_part == '{':
                # Block starts on same line
                # Find matching closing brace
                depth = 1
                block_lines = []
                j = i + 1
                # Check if there's content after { on same line
                rest_of_line = line[line.index('{') + 1:].strip()
                if rest_of_line and not rest_of_line.startswith('#'):
                    block_lines.append(rest_of_line)
                while j < len(lines) and depth > 0:
                    l = lines[j]
                    stripped_l = l.strip()
                    if not stripped_l.startswith('#'):
                        for c in l:
                            if c == '{': depth += 1
                            elif c == '}': depth -= 1
                    if depth > 0:
                        block_lines.append(l)
                    j += 1
                entries.append((key, 'block', '\n'.join(block_lines), i))
                i = j
                continue
            elif value_part.startswith('{'):
                # Block starts with { and maybe has content
                depth = 1
                block_lines = []
                # Content after the {
                rest = value_part[1:]
                if rest:
                    block_lines.append(rest)
                j = i + 1
                while j < len(lines) and depth > 0:
                    l = lines[j]
                    stripped_l = l.strip()
                    if not stripped_l.startswith('#'):
                        for c in l:
                            if c == '{': depth += 1
                            elif c == '}': depth -= 1
                    if depth > 0:
                        block_lines.append(l)
                    j += 1
                entries.append((key, 'block', '\n'.join(block_lines), i))
                i = j
                continue
            else:
                # Simple value
                entries.append((key, 'value', value_part, i))
                i += 1
        else:
            # Might be a value-only line or malformed
            i += 1
    
    return entries


def extract_keys_from_block(block_text):
    """Extract all assignment keys from block text."""
    keys = set()
    for line in block_text.split('\n'):
        m = re.match(r'^\s*([\w@._-]+)\s*=', line)
        if m:
            keys.add(m.group(1))
    return keys


class ScopeTracker:
    """Tracks scope context while walking a file."""
    
    def __init__(self, initial_scope):
        self.stack = [(initial_scope, False)]  # (scope, is_transparent)
    
    @property
    def current(self):
        return self.stack[-1][0] if self.stack else "Global"
    
    def push(self, scope, is_transparent=False):
        self.stack.append((scope, is_transparent))
    
    def pop(self):
        if len(self.stack) > 1:
            self.stack.pop()
    
    def resolve_key_scope(self, key):
        """Determine what scope a key pushes when it has a block value."""
        # Transparent blocks pass through
        if key in TRANSPARENT_KEYWORDS:
            return (self.current, True)
        
        # Known scope push keywords
        if key in SCOPE_PUSH_KEYWORDS:
            return (SCOPE_PUSH_KEYWORDS[key], False)
        
        # Chain targets
        if key in ("OWNER",):
            # OWNER from current scope: Character → Country, others stay
            if self.current == "Character":
                return ("Country", False)
            return (self.current, True)
        
        if key == "FROM":
            # FROM refers to previous non-transparent scope
            for (s, t) in reversed(self.stack[:-1]):
                if not t:
                    return (s, False)
            return ("Country", False)
        
        if key == "ROOT":
            # ROOT is first non-transparent scope
            for (s, t) in self.stack:
                if not t:
                    return (s, False)
            return ("Global", False)
        
        if key == "PREV":
            # PREV is previous non-transparent before current
            found_current = False
            for (s, t) in reversed(self.stack):
                if not found_current:
                    if not t:
                        found_current = True
                    continue
                if not t:
                    return (s, False)
            return ("Global", False)
        
        if key == "THIS":
            return (self.current, True)
        
        # Unknown key → stays at current scope
        return (None, False)  # None means "don't push"
    
    def process_block_keys(self, block_text):
        """
        Walk through a block's entries, tracking scope and recording
        which triggers/effects appear at which scope levels.
        Returns list of (key, scope) tuples.
        """
        results = []
        entries = parse_script_lines(block_text.split('\n'))
        
        for key, value_type, value_text, line_no in entries:
            if value_type == 'block':
                # Determine what scope this key pushes
                pushed_scope, is_transparent = self.resolve_key_scope(key)
                
                if pushed_scope is not None:
                    self.push(pushed_scope, is_transparent)
                    results.extend(self.process_block_keys(value_text))
                    self.pop()
                else:
                    # Unknown key with block — process children at same scope
                    # but don't push the key itself
                    results.extend(self.process_block_keys(value_text))
            else:
                # Value assignment — record the key at current scope
                # Skip known structural/block keys
                if key not in TRANSPARENT_KEYWORDS and key not in SCOPE_PUSH_KEYWORDS:
                    if not key.startswith('#'):
                        results.append((key, self.current))
        
        return results


def analyze_vanilla_files(vanilla_dir):
    """
    Walk all vanilla .txt files and record trigger/effect usage by scope.
    """
    # Results: entity_name -> {scope: count}
    entity_scope_counts = defaultdict(lambda: defaultdict(int))
    # Track which files were analyzed
    files_analyzed = 0
    entries_processed = 0
    
    # Find all .txt files in the vanilla directory recursively
    for root, dirs, files in os.walk(vanilla_dir):
        for fname in files:
            if not fname.endswith('.txt'):
                continue
            
            filepath = os.path.join(root, fname)
            relpath = os.path.relpath(filepath, vanilla_dir)
            
            # Skip mod files and non-game files
            if '/.git/' in filepath or '/.svn/' in filepath:
                continue
            
            try:
                with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
                    content = f.read()
            except Exception:
                continue
            
            # Get initial scope for this file
            initial_scope = get_initial_scope(relpath)
            
            # Parse top-level entries
            lines = content.split('\n')
            entries = parse_script_lines(lines)
            
            tracker = ScopeTracker(initial_scope)
            
            for key, value_type, value_text, line_no in entries:
                if value_type == 'block':
                    pushed_scope, is_transparent = tracker.resolve_key_scope(key)
                    
                    if pushed_scope is not None:
                        tracker.push(pushed_scope, is_transparent)
                        scope_results = tracker.process_block_keys(value_text)
                        tracker.pop()
                    else:
                        scope_results = tracker.process_block_keys(value_text)
                else:
                    if key not in TRANSPARENT_KEYWORDS and key not in SCOPE_PUSH_KEYWORDS:
                        if not key.startswith('#'):
                            scope_results = [(key, tracker.current)]
                        else:
                            scope_results = []
                    else:
                        scope_results = []
                
                for entity_key, scope in scope_results:
                    entity_scope_counts[entity_key][scope] += 1
                    entries_processed += 1
            
            files_analyzed += 1
            
            if files_analyzed % 500 == 0:
                print(f"  ... analyzed {files_analyzed} files, {entries_processed} entries")
    
    return entity_scope_counts, files_analyzed, entries_processed


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Analyze vanilla HOI4 file scopes")
    parser.add_argument("vanilla_dir", help="Path to HOI4 game directory")
    parser.add_argument("--output", "-o", default="vanilla_scope_analysis.json",
                        help="Output JSON file path")
    parser.add_argument("--min-count", type=int, default=1,
                        help="Minimum usage count to include in output")
    args = parser.parse_args()
    
    vanilla_dir = os.path.expanduser(args.vanilla_dir)
    if not os.path.isdir(vanilla_dir):
        print(f"Error: {vanilla_dir} is not a directory")
        return 1
    
    print(f"Analyzing vanilla files in: {vanilla_dir}")
    print("This may take a while for large game directories...")
    
    entity_scope_counts, files_analyzed, entries_processed = analyze_vanilla_files(vanilla_dir)
    
    print(f"\nAnalyzed {files_analyzed} files, {entries_processed} entries")
    print(f"Found {len(entity_scope_counts)} unique entities")
    
    # Convert to serializable format
    output = {
        "metadata": {
            "vanilla_dir": vanilla_dir,
            "files_analyzed": files_analyzed,
            "entries_processed": entries_processed,
            "entity_count": len(entity_scope_counts),
        },
        "entities": {}
    }
    
    for entity, scope_counts in sorted(entity_scope_counts.items()):
        total = sum(scope_counts.values())
        if total < args.min_count:
            continue
        
        # Sort scopes by count (descending)
        sorted_scopes = sorted(scope_counts.items(), key=lambda x: -x[1])
        output["entities"][entity] = {
            "total": total,
            "scopes": dict(sorted_scopes),
        }
    
    with open(args.output, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"\nOutput written to: {args.output}")
    
    # Summary: entities with single scope
    single_scope = {e: d for e, d in output["entities"].items()
                    if len(d["scopes"]) == 1}
    multi_scope = {e: d for e, d in output["entities"].items()
                   if len(d["scopes"]) > 1}
    
    print(f"\nEntities with single scope: {len(single_scope)}")
    print(f"Entities with multiple scopes: {len(multi_scope)}")
    
    # Print entities where >90% usage is in one scope
    print(f"\n=== High-confidence single-scope entities (>90% in one scope) ===")
    for entity, data in sorted(output["entities"].items()):
        total = data["total"]
        for scope, count in data["scopes"].items():
            ratio = count / total
            if ratio >= 0.9:
                print(f"  {entity}: {scope} ({count}/{total}, {ratio:.0%})")
                break
    
    return 0


if __name__ == "__main__":
    main()
