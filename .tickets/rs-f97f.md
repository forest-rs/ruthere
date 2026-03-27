---
id: rs-f97f
status: closed
deps: []
links: []
created: 2026-03-27T16:34:03Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add server ingress and watch traits

Add a narrow trait layer in ruthere_server that captures the current server ingest and watcher-polling contracts without introducing transport or auth abstractions.

## Design

Keep the trait layer small and concrete. Split it into write-side ingest/lifecycle and watcher-polling contracts that PresenceServer already satisfies. Do not abstract over the richer read-model surface yet; keep store-backed snapshot/projection queries on the concrete server. Add tests that use generic helpers against the traits so the contract is pressure-tested without inventing a network API.

## Acceptance Criteria

Add public server trait(s) with rustdoc and implement them for PresenceServer; document the boundary in a new or updated crate-local ADR; update README and, where helpful, the watcher example or tests to demonstrate the traits; keep fmt/clippy/test/doc/typos/copyright green.


## Notes

**2026-03-27T16:35:47Z**

Added a narrow trait layer to ruthere_server with PresenceIngress for update receipt/lifecycle and PresenceWatch for watcher cursor creation and retained-change polling. PresenceServer now implements both traits, the crate docs and README call out the new contracts, and ADR-0002 documents why the trait layer stops short of abstracting the richer store-backed read model. Generic helper functions in the server tests exercise the traits directly so the contract is pressure-tested without inventing transport behavior or refactoring the watcher example into a trait-heavy shape. Validation: cargo fmt --all; taplo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps; typos; bash .github/copyright.sh. All validation commands passed.
