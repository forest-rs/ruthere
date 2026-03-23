---
id: rs-mydr
status: closed
deps: []
links: []
created: 2026-03-23T04:37:49Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add incremental store change cursors

Retain store-assigned changes so callers can consume presence updates incrementally by sequence instead of polling full snapshots.

## Design

Keep sequencing and change retention in ruthere_store. Expose store change records and a changes_since API. Emit changes for both published updates and expiry removals so incremental consumers can observe disappearance as well as assertion.

## Acceptance Criteria

Document the change-log boundary in a crate-local ADR; add public change types and cursor APIs to ruthere_store; retain publish and expiry changes with monotonic sequences; update the example to demonstrate changes_since; add focused tests; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T04:39:20Z**

Implemented retained change cursors in ruthere_store. The crate now exposes StoreChange and StoreChangeKind, retains published updates and expiry-driven removals with monotonic local sequences, and provides has_changes_since and changes_since APIs for incremental consumers. The basic_presence_flow example now prints retained changes before and after expiry so the intended watcher seam is visible end-to-end. This is an additive store API change; raw snapshots and subject projections remain unchanged. Validation: cargo fmt --all; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo run -p basic_presence_flow; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
