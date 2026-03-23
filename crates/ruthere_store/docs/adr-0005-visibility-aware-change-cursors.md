# ADR-0005: Visibility-Aware Change Cursors

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rs-e347`

## Context

`ruthere_store` supports visibility-aware snapshot and projection reads, but
incremental consumers still only have unfiltered change cursors. That leaves
the watcher story structurally incomplete: a caller can ask what changed since
sequence `N`, but not "what visible changes happened since sequence `N`?"

Expiry removals are the key complication. Once an entry is removed from the
store, the cursor layer still needs enough retained metadata to evaluate
visibility safely.

## Decision

`ruthere_store` now supports visibility-aware retained change cursors.

The store:

- retains visibility labels on expiry-driven removals
- exposes `changes_since_visible`
- exposes `has_visible_changes_since`

The crate still does not own the visibility policy itself. Callers supply that
policy through the same `VisibilityPolicy` seam used by snapshot and projection
reads.

## Consequences

The watcher story is now complete within one process-local store:
publish facts, consume visible deltas, materialize visible state, and project
visible subject summaries.
