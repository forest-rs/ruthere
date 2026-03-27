# ADR-0001: `ruthere_beacon` Boundary

- Status: Accepted
- Date: 2026-03-27
- Ticket: `rb-xgra`

## Context

`ruthere_core` already defines calm update types, and `ruthere_store` now
provides watcher cursors on the read side. The write side is still repetitive:
callers repeatedly restate address, origin, visibility, and expiry details
every time they publish an update.

The next slice should improve write-side ergonomics without moving storage,
clock, or transport responsibilities into a new crate.

## Decision

`ruthere_beacon` owns publication ergonomics for one addressed presence source.

The crate:

- stores one `(address, origin)` publication target
- stores stable visibility and expiry policy
- resolves expiry for each observation timestamp
- builds ready-to-publish `PresenceUpdate` values
- provides a small common-path helper for heartbeat updates

The crate intentionally does not own:

- storage or materialization
- transport or server behavior
- wall-clock access or background timers
- automatic publishing or retry behavior

## Consequences

This gives `ruthere` a calm write-side seam that mirrors the read-side
`WatcherCursor`: one small type that removes repeated boilerplate without
inventing a runtime.
