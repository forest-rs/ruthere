---
id: rs-j2rz
status: closed
deps: []
links: []
created: 2026-03-27T16:21:26Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add ruthere_server crate

Add a small process-local server crate that receives PresenceUpdate values and composes ruthere_store behind a server-oriented runtime seam.

## Design

Keep the first slice no_std and intentionally local. The server should own ingest, expiry application, and watcher polling against its internal store, but it should not introduce transport, auth, or background fanout. Expose the underlying store read-only so callers can still reach the richer materialization and projection APIs without duplicating the entire store surface.

## Acceptance Criteria

Add crates/ruthere_server as a workspace crate with README and rustdoc; document the boundary in a crate-local ADR; provide a public PresenceServer API with receive/expire/cursor helpers plus focused tests; update the watcher example to use the server; keep fmt/clippy/test/doc/typos/copyright green.


## Notes

**2026-03-27T16:25:22Z**

Implemented a new no_std crate at crates/ruthere_server as a small process-local ingest/runtime seam over ruthere_store. PresenceServer owns receive/receive_all, expiry application, watcher cursor creation, and watcher polling helpers against its internal store, while exposing the underlying store read-only for richer snapshot and projection queries. The boundary is documented in ADR-0001, the crate README includes a small example, and watcher_presence_flow now publishes beacons into a PresenceServer instead of ingesting directly into InMemoryStore so the client/server/store topology is explicit. Validation: cargo fmt --all; cargo run -p watcher_presence_flow; cargo run -p basic_presence_flow; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
