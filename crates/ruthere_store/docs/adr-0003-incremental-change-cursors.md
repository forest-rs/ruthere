# ADR-0003: Incremental Change Cursor Boundary

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rs-mydr`

## Context

`ruthere_store` can already materialize current snapshots and projected subject
summaries, but callers still need to poll those views to notice updates. Real
watchers usually want an incremental seam: "what changed since sequence N?"

The store already owns monotonic local sequencing, so the next step is to
retain store-assigned changes and expose cursor-based reads without introducing
transport or push delivery.

## Decision

`ruthere_store` now retains a local change log and exposes cursor reads over it.

The change log:

- is assigned by the store's monotonic local sequence
- records both published updates and expiry-driven removals
- is queryable with `changes_since(sequence)`
- supports cheap checks via `has_changes_since(sequence)`

The change log intentionally does not yet own:

- watcher registration
- push delivery or callback APIs
- durability or replay across process boundaries
- retention/compaction policies beyond in-memory growth

## Change Rules

1. Every published update appends one `Published` change.
2. Every expiry-driven removal appends one `Expired` change.
3. Sequences are strictly increasing within one store instance.
4. `changes_since(N)` returns all retained changes with `sequence > N`.

## Consequences

This gives `ruthere` its first complete non-transport watcher seam:
publish facts, consume deltas, materialize state, and project subject views.
Later slices can add subscriptions or retention controls on top of the same
sequence model.
