---
id: bpf-301w
status: closed
deps: []
links: []
created: 2026-03-23T03:54:37Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add basic presence flow example crate

Add a top-level example crate that demonstrates the intended end-to-end ruthere flow with concrete key types and an in-memory store.

## Design

Keep the example narrow and executable: publish presence from multiple resources, inspect snapshots, and expire stale entries. Use it to pressure-test API ergonomics rather than introduce new library abstractions.

## Acceptance Criteria

Create examples/basic_presence_flow as a workspace crate with a README and runnable main; show concrete key types, publication, snapshot lookup, context materialization, and expiry; update workspace/CI wiring so checks remain green; record why no ADR is needed for the example-only crate; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T03:56:35Z**

Implemented examples/basic_presence_flow as a runnable top-level workspace crate. The example uses concrete subject, context, resource, and origin key types to show the intended end-to-end flow: publish multiple updates, inspect one snapshot, materialize all snapshots in a context, and prune stale entries with expiry. CI/workspace wiring was updated to exclude this std-only binary crate from the no_std job. No ADR was added because this change does not introduce a new architectural boundary or semantic policy; it is an onboarding/example slice over existing core and store behavior. Validation: cargo fmt --all; taplo fmt; bash .github/copyright.sh; cargo run -p basic_presence_flow; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos. All validation commands passed.
