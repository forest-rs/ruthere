---
id: rc-5j1i
status: closed
deps: []
links: []
created: 2026-03-23T04:02:23Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# add convenience helpers for presence updates

Reduce ceremony in the common builder path for built-in facets so example and store call sites read like intended usage rather than enum plumbing.

## Design

Add calm convenience methods to PresenceUpdate for built-in and extension facets, with focused built-in setters and clearers for the common path. Keep the underlying typed facet model unchanged.

## Acceptance Criteria

Implement documented helper methods in ruthere_core; update examples and tests to use the calmer API; keep fmt/clippy/test/doc/typos green.


## Notes

**2026-03-23T04:04:13Z**

Added convenience helpers to PresenceUpdate and PresenceSnapshot to reduce enum-plumbing at common call sites. New update helpers cover set_builtin, clear_builtin, set_extension, clear_extension, and focused built-in helpers for availability, activity, and last_seen plus matching clearers. The example and store/core tests were updated to use the calmer builder path. Validation: cargo fmt --all; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo run -p basic_presence_flow; cargo doc --no-deps; bash .github/copyright.sh; typos. All validation commands passed.
