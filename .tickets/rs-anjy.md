---
id: rs-anjy
status: closed
deps: []
links: []
created: 2026-03-27T03:11:26Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add store watcher cursors

Add a small sequence-tracking watcher abstraction over the existing retained change cursor APIs in ruthere_store.

## Design

Keep the abstraction local and synchronous. The cursor should own only store sequence position and offer calm helper methods for unfiltered and visibility-aware retained change polling. Do not add watcher identity, push delivery, async behavior, or transport semantics. Update the watcher example to use the new abstraction so the intended flow becomes simpler and more coherent.

## Acceptance Criteria

Add a public watcher/cursor type in ruthere_store with rustdoc; cover unfiltered and visibility-aware polling with tests; update crate docs/README and the watcher example to use it; add or update a crate-local ADR for the boundary; keep fmt/clippy/test/doc/typos/copyright green.


## Notes

**2026-03-27T03:15:55Z**

Implemented a local WatcherCursor abstraction in ruthere_store to own retained change sequence position without introducing watcher identity, async behavior, or push delivery. The new public type lives in src/cursor.rs, is re-exported from the crate root, and provides unfiltered and visibility-aware pending/poll helpers over the existing retained change log. The watcher example now uses WatcherCursor instead of hand-managed u64 cursor state, and the store README plus ADR-0006 document the new boundary. Added unit coverage for unfiltered and visibility-aware watcher polling semantics. Validation: cargo fmt --all; cargo run -p watcher_presence_flow; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
