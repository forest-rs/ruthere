# ADR-0001: `ruthere_core` Presence Boundary

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rc-plgu`

## Context

`ruthere` is intended to be a small presence substrate, not a transport stack, policy engine, or UI system.
The first crate in the workspace must therefore establish a fence that keeps the durable semantic model small and
stable while leaving runtime behavior free to evolve in later crates.

The design sketch raised several open questions around watcher identity, subscriptions, sequencing, extension
facets, visibility, and snapshot materialization. The immediate risk is letting `ruthere_core` absorb store or
policy concerns before the semantic model is proven.

## Decision

`ruthere_core` owns the vocabulary for presence assertions and the types needed to reduce assertions into current
state. It does not own watcher identity, subscription delivery, store sequencing, or visibility evaluation.

The initial public surface uses:

- generic keys supplied by the embedding application
- closed built-in facets for the common path
- a generic extension facet slot for caller-defined facts
- explicit timestamps and expiry labels
- visibility labels that are carried, but not interpreted, by the core
- updates that are scoped to one subject/context/resource origin at a time
- snapshots that preserve both built-in and extension facets

The initial public surface intentionally does not include:

- watcher identifiers
- transport-neutral wire models
- store traits
- subscription cursors
- logical clocks or replication semantics
- policy traits for evaluating visibility labels

## Invariants

1. `ruthere_core` stays `no_std` with `alloc`.
2. Presence is modeled as typed facets, not a monolithic status blob.
3. Resource is part of the addressed scope of an update, not itself a facet.
4. Visibility in core is a label, not a policy decision.
5. Updates are append-like semantic inputs; sequencing belongs to stores.
6. Snapshots retain extension facets rather than flattening down to built-ins only.

## Consequences

The first runtime crate, expected to be `ruthere_store`, will own:

- in-memory indexing
- publication APIs
- sequencing
- conflict resolution across origins/resources
- subscription registration and delivery
- expiry execution
- bulk snapshot materialization

This keeps `ruthere_core` calm and reusable while allowing runtime behavior to evolve from real usage rather than
premature abstraction.
