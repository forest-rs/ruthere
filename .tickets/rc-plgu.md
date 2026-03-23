---
id: rc-plgu
status: closed
deps: []
links: []
created: 2026-03-23T03:07:10Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# establish ruthere_core presence model v1

Define the initial no_std + alloc semantic model for presence assertions, updates, snapshots, timestamps, expiry, and visibility labels in ruthere_core.

## Design

Keep ruthere_core as a pure semantic crate. Exclude watcher identity, sequencing, subscriptions, and policy evaluation. Use closed built-in facets plus a generic extension slot.

## Acceptance Criteria

Document the core/store boundary in a crate-local ADR; expose a minimal documented public API for keys, timestamps, expiry, visibility labels, built-in facts, updates, and snapshots; add tests covering core invariants; keep cargo fmt/clippy/test green.


## Notes

**2026-03-23T03:14:04Z**

Implemented the initial ruthere_core semantic model and ADR.

Decision summary:
- keep core as no_std + alloc
- carry visibility labels but do not evaluate them
- keep watcher identity, sequencing, subscriptions, and store traits out of core
- model built-in facets as a closed enum plus a generic extension facet slot
- scope updates to one subject/context/resource/origin batch
- preserve extension facets in snapshots

Validation:
- cargo fmt --all
- taplo fmt
- bash .github/copyright.sh
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo test --workspace --all-features
- cargo doc --no-deps
- typos

All validation commands passed.
