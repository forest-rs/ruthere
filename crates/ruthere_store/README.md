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
change log without introducing push delivery semantics. Retained polling is now
gap-aware: `changes_since`, `changes_since_visible`, and watcher polling return
explicit gap metadata when compaction has moved past a caller cursor.

Filtered read APIs let callers apply their own visibility policy to snapshots
and subject summaries without moving that policy into `ruthere_core`.
Filtered cursor APIs now apply that same visibility policy to retained changes.

The store treats per-resource, per-origin snapshots as the canonical truth.
`SubjectPresenceSummary` is a derived projection over those snapshots rather
than a replacement for them.

When polling returns a retained gap, the caller must rebuild from current
materialized state before advancing its cursor. Moving a cursor alone does not
reconstruct lost changes.

Future store work may still revisit explicit materialized-view revision or
projection freshness once concrete consumers justify them. See
[ADR-0007](docs/adr-0007-resource-first-presence-roadmap.md) and
[ADR-0010](docs/adr-0010-retained-log-compaction-and-resync.md).

The crate does not yet define watcher identity, push subscriptions, watcher
policy evaluation, or transport integration.
