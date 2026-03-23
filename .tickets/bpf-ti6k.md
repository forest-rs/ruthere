---
id: bpf-ti6k
status: closed
deps: []
links: []
created: 2026-03-23T05:38:48Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add watcher presence flow example crate

Add a second top-level example crate that demonstrates the watcher-oriented ruthere flow using retained change cursors, visibility filtering, and subject summaries.

## Design

Keep the example std-only, executable, and focused on one watcher loop. Show how a client tracks last_seen_sequence, polls visible changes, refreshes projected subject summaries, and reacts to expiry. Use it to pressure-test whether the existing cursor APIs are calm enough before adding a higher-level subscription wrapper.

## Acceptance Criteria

Create examples/watcher_presence_flow as a workspace crate with a README and runnable main; show publication, last_seen_sequence tracking, visibility-filtered changes_since polling, summary refresh, and expiry reactions; update workspace/CI wiring if needed; no ADR needed for the example-only crate if the existing ADR set already covers semantics; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T05:42:55Z**

Implemented examples/watcher_presence_flow as a second runnable top-level example crate focused on the watcher loop. The example uses two viewer policies, explicit last_seen_sequence tracking, has_visible_changes_since/changes_since_visible polling, and visibility-filtered subject summary refreshes to show how callers can consume retained change cursors today without a higher-level subscription wrapper. Workspace membership and CI std-only exclusions were updated for the new binary crate, and the example README documents the intended flow and NO_COLOR behavior. No ADR was added because this change does not introduce a new architectural boundary or semantic policy; it is an example-only slice over existing store semantics already covered by the current ADR set. Validation: cargo fmt --all; cargo run -p watcher_presence_flow; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
