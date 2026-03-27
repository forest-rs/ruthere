# ADR-0001: `ruthere_server` Boundary

- Status: Accepted
- Date: 2026-03-27
- Ticket: `rs-j2rz`

## Context

`ruthere_beacon` now gives clients a calm way to build updates, and
`ruthere_store` already provides local sequencing, materialization, and watcher
polling. What is still missing is an explicit process-local runtime seam that
looks like a server: receive updates, keep current state, expire stale entries,
and let watchers poll the resulting log.

The first slice should stay intentionally local and avoid transport or auth.

## Decision

`ruthere_server` owns a small process-local ingest/runtime seam over
`ruthere_store`.

The crate:

- receives `PresenceUpdate` values
- applies expiry against an internal store
- creates watcher cursors
- lets watchers poll retained changes through the server
- exposes the underlying store read-only for richer query operations

The crate intentionally does not own:

- network transport
- authentication or authorization
- push delivery or background fanout
- clock sources or background scheduling

## Consequences

This makes the client/server/store topology explicit without inventing a full
network stack. Callers can now point beacons at a server-shaped API, while the
store remains the source of truth for local state and projections.
