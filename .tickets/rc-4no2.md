---
id: rc-4no2
status: closed
deps: []
links: []
created: 2026-04-09T16:21:20Z
type: task
priority: 2
assignee: Bruce Mitchener
---
# Broaden resource semantics and add associated-resource example

Update ruthere_core documentation so resource and origin are not described as only subject-owned devices, and add a runnable example that pressure-tests associated resources such as trackers and shipments without changing the core API.

## Design

ruthere_core owns semantic scope, not ownership semantics. Broaden resource/origin docs and the core README, update ADR-0007 to describe associated contributors, and add a top-level example crate that models one subject across direct and associated resources using typed extension facets defined in the example crate.

## Acceptance Criteria

Core docs describe resource as an optional contributor to a subject's presence within a context and origin as provenance; ADR-0007 reflects the broader mental model; a new runnable example demonstrates direct and associated resources with typed extension facets; workspace validation passes.


## Notes

**2026-04-09T16:24:35Z**

Broadened ruthere_core rustdocs and README so resource means an optional contributor to a subject's presence within one context, not only a subject-owned endpoint, and origin is documented as provenance rather than ownership. Updated ADR-0007 to describe associated resources explicitly. Added the runnable example crate examples/associated_resource_flow with example-only typed extension facets for subject-resource relationship, resource class, delivery status, and tracker location; it demonstrates one subject across a direct laptop resource, an associated tracker resource, and an external shipment resource without changing the core API. Validation: typos; taplo fmt; cargo fmt --all; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --no-deps.
