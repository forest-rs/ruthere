---
id: rb-xgra
status: closed
deps: []
links: []
created: 2026-03-27T15:30:16Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add ruthere_beacon crate

Add a small no_std write-side crate that owns publication ergonomics for one addressed presence source.

## Design

Keep the first slice narrow and synchronous. The crate should configure one address/origin/visibility/lease policy and build ready-to-publish PresenceUpdate values without owning storage, clocks, scheduling, or transport. Favor common helpers for heartbeat, availability, and activity over a general runtime abstraction. Include rustdoc/tests that pressure-test the call-site.

## Acceptance Criteria

Add crates/ruthere_beacon as a workspace crate with README and rustdoc; document the crate boundary in a crate-local ADR; provide a public PresenceBeacon API plus lease/expiry helpers and focused tests; keep the crate no_std without new dependencies; keep fmt/clippy/test/doc/typos/copyright green.


## Notes

**2026-03-27T15:36:08Z**

Implemented a new no_std write-side crate at crates/ruthere_beacon. The crate defines PresenceBeacon, which captures stable publication metadata for one addressed source and builds PresenceUpdate values with configured visibility and relative expiry policy, plus a heartbeat_at helper for the common last-seen refresh path. The boundary is documented in ADR-0001, the crate README includes a runnable call-site, and the basic_presence_flow and watcher_presence_flow examples now use beacons for publication so the new seam is exercised in real examples. To keep the common path inference-friendly, PresenceBeacon::new is specialized for the default Never extension type and PresenceBeacon::new_typed is available when callers want an explicit extension facet type. Validation: cargo fmt --all; cargo run -p basic_presence_flow; cargo run -p watcher_presence_flow; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
