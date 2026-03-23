---
id: rs-4tm4
status: closed
deps: []
links: []
created: 2026-03-23T04:56:56Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add visibility-aware store reads

Allow callers to filter snapshots and projected subject summaries through a caller-supplied visibility policy while keeping visibility semantics out of ruthere_core.

## Design

Add a visibility policy trait to ruthere_store, implement it for closures, and add filtered read/projection APIs. Keep the change log unchanged in this slice; expiry change filtering would require carrying visibility through removals.

## Acceptance Criteria

Document the visibility filtering boundary in a crate-local ADR; add public visibility policy support and filtered snapshot/projection APIs; update the example to show different viewer perspectives; add focused tests; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T04:58:45Z**

Implemented visibility-aware reads in ruthere_store. The crate now exposes a closure-friendly VisibilityPolicy trait plus filtered snapshot and subject-summary APIs. Raw reads remain unchanged; callers can now ask for snapshot_visible, snapshots_in_context_visible, subject_summary_in_context_visible, and subject_summaries_in_context_visible without moving policy into ruthere_core. The change log is intentionally unchanged in this slice because expiry removals do not yet retain enough visibility metadata for safe filtering. The example now shows doc-member and public-only viewer perspectives over the same underlying state. Validation: cargo fmt --all; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo run -p basic_presence_flow; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
