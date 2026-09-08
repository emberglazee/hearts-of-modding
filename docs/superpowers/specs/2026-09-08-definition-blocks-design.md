# Parent-Keyed Definition Blocks — Design Spec

**Date:** 2026-09-08
**Status:** approved §§1–4 by Embi (chat, 2026-09-08)
**Goal:** Replace the misclassification of definition-site container keys
(`activation`, `available`, `visible`, …) as global triggers/effects with a
parent-keyed data model, ending the impurity class without replaying past
table-migration incidents.

## Background

`hoi4_data.json` (`triggers`/`effects`/`modifiers`) holds three impurity
classes: ~80 scope iterators (valid statements, harmless), 21 transparent
wrappers (already have their own list), and ~15 **definition-site containers**
valid only as children of specific definition blocks. That third class is
suggested by completion in blocks where it is meaningless (`activation`
inside event options) and cannot be fixed by tagging: `available` means
different things in a focus, a technology, and a decision — validity is
parent-relative, and only parent-keyed storage expresses that.

Precedent: schema-v3 per-entity `parameters` (325 entities / 1021 params)
already does parent-keyed storage for static entities. This design
generalizes it to file-type/dynamic parents (decisions, missions).

## §1 Data schema

New top-level `definition_blocks` map in `hoi4_data.json`:

```json
"definition_blocks": {
    "decision": {
        "match": { "paths": ["common/decisions/"], "kinds": ["decision", "mission"] },
        "params": { "activation": { …ParameterDef…, "scope_push": "Country",
                                    "body": "triggers" }, … }
    }
}
```

- Reuses `ParameterDef`; adds optional `scope_push` (scope the block pushes)
  and `body` (`triggers` | `effects` | `both` — what the block's body holds).
- **Removal criterion (per key):** a key leaves `triggers`/`effects` only
  when (a) every parent it occurs under carries it, and (b) a vanilla-corpus
  scan shows zero hits outside those parents. Additive first, delete last.
- Conflict rule: file path beats ancestor key when they disagree.
- `decision` and `mission` share one kind entry (same files, near-identical
  params; mission-only keys are `optional`). No per-block kind detection.
- Untouched: iterators (stay as statements), `transparent_block_types`,
  modifiers table.

## §2 Consumer contracts

`definition_blocks` is consulted by parent context first; global tables never
suggest or validate definition params.

| Consumer | Contract |
|---|---|
| Completion | Chain-walk gains file-path fallback for unknown instance keys; kind params offered with `0_` sort_text. Generic scope list excludes keys present in any kind's params. |
| Semantic tokens | Key is `Keyword` if global OR in enclosing kind's params (highlighting stays forgiving). |
| Hover | Falls back to parent-context param lookup; shows parent-specific description. |
| Scope resolver | Checks enclosing kind's param `scope_push` before global tables. |
| V2ScopeRule | Bodies validated per param `body`; param keys skipped like `pushes_scope` entities. |
| `parse_official_docs.py --add-only` | Skips doc entities matching any kind param name (no re-pollution). |

## §3 Migration

- **Phase 0:** schema + Rust types/accessors, empty maps. Suite green.
- **Phase 1 (pilot):** `decision` kind params; wire four read consumers with
  global fallback kept. Gate: zero diagnostic delta on vanilla +
  Hearts-of-Minecraft decisions.
- **Phase 2:** per-key removal via the §1 criterion. `activation` first
  (single parent); `available`/`visible` last (three parents each).
- **Phase 3:** generator skip-list + CI check failing on dual presence
  unless in a shrink-only grandfathered-exceptions array.
- Non-goals: iterator unification, wrapper list, modifiers.

## §4 Tests

1. Per-key contract tests (parent completion in, global completion out,
   scope push, self-silence), each proven to bite pre-migration.
2. Corpus parity gate (vanilla + HoM decisions before/after diff).
3. Generator idempotency (second `--add-only` run is a no-op).
4. Ratchet test (exceptions array never grows).
5. Standard gate per phase: `cargo test --bin hom-lsp`,
   `clippy --all-targets -- -D warnings`, `fmt --check`; one commit/phase.
