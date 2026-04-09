# ruthere_store

`ruthere_store` is the first runtime crate for `ruthere`.

It provides an in-memory store over `ruthere_core` updates and snapshots. The
crate owns sequencing, snapshot reduction, and expiry pruning for the local
store.

It also provides a subject-level projection layer that can turn multiple
resource snapshots into a calm subject summary while preserving the underlying
resource detail.

It now also retains a local change log so callers can consume incremental
changes with store-assigned cursors.
`WatcherCursor` provides a calmer local watcher abstraction over that retained
change log without introducing push delivery semantics.

Filtered read APIs let callers apply their own visibility policy to snapshots
and subject summaries without moving that policy into `ruthere_core`.
Filtered cursor APIs now apply that same visibility policy to retained changes.

The store treats per-resource, per-origin snapshots as the canonical truth.
`SubjectPresenceSummary` is a derived projection over those snapshots rather
than a replacement for them.

Future store work is expected around explicit materialized-view revision,
freshness distinct from availability, and eventual gap or resync semantics for
retained changes. See
[ADR-0007](docs/adr-0007-resource-first-presence-roadmap.md).

The crate does not yet define watcher identity, push subscriptions, watcher
policy evaluation, or transport integration.
