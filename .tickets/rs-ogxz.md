---
id: rs-ogxz
status: closed
deps: []
links: []
created: 2026-03-23T04:18:16Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add subject-level presence projection

Project raw per-resource presence snapshots into subject-level summaries with resource detail so callers can render one person-level view without losing underlying resource state.

## Design

Keep projection above raw store entries and below UI. Add a dedicated projection module in ruthere_store with summary types and a default dominance policy. Raw snapshot APIs remain unchanged.

## Acceptance Criteria

Document the projection boundary in a crate-local ADR; add subject-summary APIs to ruthere_store with a default policy; preserve resource detail in the summary output; update the example to print both raw snapshots and projected subject summaries; add focused tests; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T04:23:07Z**

Implemented subject-level projection in ruthere_store. The new projection module adds SubjectPresenceSummary, a SubjectProjectionPolicy trait, and DefaultSubjectProjectionPolicy. InMemoryStore now exposes subject_summary_in_context and subject_summaries_in_context, with _with_policy variants for custom aggregation rules. The summary keeps resource-level snapshots while projecting a dominant resource, origin, availability, activity, last_seen, and observed_at headline. The basic_presence_flow example now prints both raw snapshots and projected subject summaries. This is an additive public API change; raw snapshot APIs remain unchanged. Validation: cargo fmt --all; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo run -p basic_presence_flow; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
