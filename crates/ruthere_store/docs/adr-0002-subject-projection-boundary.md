# ADR-0002: Subject-Level Projection Boundary

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rs-ogxz`

## Context

`ruthere_store` already materializes current per-resource snapshots, but callers
often need a subject-level view for rendering presence. A user with multiple
active resources should not require every caller to manually aggregate
contradictory or overlapping resource facts in UI code.

At the same time, raw resource snapshots are still valuable and should remain
available. The next slice needs to add subject-level projection without hiding
or rewriting the underlying facts.

## Decision

`ruthere_store` owns subject-level projection as a separate layer above raw
stored entries.

The crate now provides:

- a projected subject summary type
- a default dominance policy for choosing the primary resource-level signal
- store methods that return subject summaries in a context
- summary outputs that preserve the underlying resource snapshots

The projection layer intentionally does not yet own:

- visibility evaluation against observers
- UI formatting or display strings
- subscription fanout
- distributed merge or reconciliation across stores

## Projection Rules

1. Raw snapshots remain the source material and are still directly accessible.
2. Subject summaries group snapshots by `(subject, context)`.
3. The default policy picks a dominant resource signal by activity first, then
   availability, then recency.
4. The summary headline fields are taken from the dominant resource when
   present, with per-field fallback to the best available resource signal.
5. Resource detail is preserved as a vector of underlying snapshots.

## Consequences

This gives callers a calm subject-level API without forcing UI code to invent
its own aggregation semantics. The store remains responsible for projection, and
later slices can add visibility-aware filtering or incremental projections on
top of the same seam.
