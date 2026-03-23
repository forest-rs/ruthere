---
id: rs-gsi2
status: closed
deps: []
links: []
created: 2026-03-23T03:35:41Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add ruthere_store in-memory presence store

Add the first runtime crate for publishing presence updates, materializing scoped snapshots, and expiring stale entries in memory.

## Design

Keep ruthere_store responsible for sequencing, indexing, snapshot reduction, and expiry execution. Do not pull watcher identity or transport concerns into the crate yet.

## Acceptance Criteria

Create a new crate with crate-local docs and README; implement an in-memory store over ruthere_core updates and snapshots; support publish, per-address snapshot lookup, bulk context snapshot materialization, and expiry pruning; add focused tests; keep fmt/clippy/test green.


## Notes

**2026-03-23T03:39:03Z**

Implemented the first ruthere_store runtime crate. Decision summary: store entries are keyed by address plus origin; publish order is authoritative within one store and recorded by a monotonic sequence; publish updates replace entry metadata and upsert or clear facets by facet kind; bulk reads materialize snapshots by address or by context; expiry prunes whole entries rather than individual facets. Validation: cargo fmt --all; taplo fmt; bash .github/copyright.sh; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos. All validation commands passed.
