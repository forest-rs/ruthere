# ADR-0002: Server Ingress and Watch Traits

- Status: Accepted
- Date: 2026-03-27
- Ticket: `rs-f97f`

## Context

`ruthere_server` now provides a concrete process-local server, but future
networked or embedded runtimes will likely want to implement only a narrow
contract instead of exposing the concrete type directly.

The first trait layer should stay small. It should describe the current server
seams that are already real in the codebase without abstracting the richer
store-backed read model or inventing transport semantics.

## Decision

`ruthere_server` now exposes two narrow traits:

- `PresenceIngress` for update receipt, sequencing, and expiry lifecycle
- `PresenceWatch` for watcher cursor creation and retained-change polling

The traits intentionally do not include:

- transport framing
- authentication or authorization
- push delivery
- snapshot or projection queries
- store ownership or replacement

Those richer query operations remain on the concrete `PresenceServer` through
its read-only `store()` accessor.

## Consequences

Future runtimes now have a calm contract to target without forcing the entire
system behind one trait object or abstracting more than the codebase has
earned. The concrete server remains the place for store-backed query behavior.
