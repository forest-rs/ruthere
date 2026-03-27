# ADR-0006: Local Watcher Cursor Boundary

- Status: Accepted
- Date: 2026-03-27
- Ticket: `rs-anjy`

## Context

`ruthere_store` already retains a local change log with raw and visibility-aware
cursor reads. That seam is semantically complete, but the call-site is still
too manual: every watcher needs to hold a sequence number, ask whether changes
exist, fetch them, and then advance its own cursor.

The examples now make that duplication obvious. The next slice should make
watcher polling calmer without inventing watcher identity, push delivery, or
transport semantics.

## Decision

`ruthere_store` now owns a local `WatcherCursor` abstraction over retained
changes.

The cursor:

- owns only a store sequence position
- supports unfiltered polling
- supports visibility-aware polling
- advances only when it returns retained changes

The cursor intentionally does not own:

- watcher identity
- visibility policy
- push delivery or async callbacks
- filtering by subject or context
- transport or network protocol behavior

## Cursor Rules

1. A new watcher cursor starts at sequence `0`.
2. A watcher cursor may also start from an explicit existing sequence.
3. Polling returns retained changes with `sequence > cursor.sequence`.
4. After a non-empty poll, the cursor advances to the newest returned
   sequence.
5. After an empty poll, the cursor position does not change.

## Consequences

This keeps the retained change log as the source of truth while giving callers
one calm, reusable way to consume it. Higher-level crates can still build push
delivery or transport-aware subscriptions later without needing to re-invent
local cursor semantics.
