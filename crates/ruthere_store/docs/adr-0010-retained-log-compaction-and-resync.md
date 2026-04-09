# ADR-0010: Retained Log Compaction and Gap Signaling

- Status: Accepted
- Date: 2026-04-09
- Ticket: `rs-x2yo`

## Context

`ruthere_store` already had retained changes, visibility-aware polling, and a
local `WatcherCursor`, but retained changes grew without bound and incremental
polling implicitly assumed the entire change log was still present.

That became structurally dishonest once we started thinking about real
retention controls. A cursor that asks for "changes since sequence 3" cannot be
answered exactly if the store has compacted away part of that range.

## Decision

`ruthere_store` now owns retained-log floor, compaction, and explicit gap
signals.

The store now provides:

- `retained_floor_sequence()` for the oldest exact cursor position
- `compact_changes_through(sequence)` to compact retained changes
- `RetainedGap` and `RetainedStatus`
- gap-aware `changes_since` and `changes_since_visible` results

`WatcherCursor` now provides:

- `status` / `status_visible`
- gap-aware `poll` / `poll_visible`

## Rules

1. The store owns in-memory retention and compaction metadata.
2. A retained gap is not silently treated as "no changes".
3. Cursor poll methods do not advance on gap.
4. Rebuilding baseline state after a gap is explicit and caller-controlled.
5. The store still does not own durable replay or transport-level
   resubscription.

## Consequences

Incremental consumers now have one honest answer when retention has moved past
their cursor: a baseline rebuild is required before incremental polling can
continue safely. Higher-level runtimes can decide how to rebuild that baseline,
but the store no longer pretends that moving a cursor alone reconstructs lost
state.
