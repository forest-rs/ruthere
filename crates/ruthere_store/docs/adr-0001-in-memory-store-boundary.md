# ADR-0001: `ruthere_store` In-Memory Store Boundary

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rs-gsi2`

## Context

`ruthere_core` now defines the semantic vocabulary for presence facts, but it
deliberately leaves runtime behavior unresolved. The first runtime crate needs
to establish ownership for sequencing, indexing, snapshot reduction, and expiry
without prematurely committing to subscriptions or transport semantics.

## Decision

`ruthere_store` owns the first in-memory runtime for `ruthere`.

The crate:

- stores presence state per `(address, origin)` entry
- assigns a monotonic store sequence to each published update
- reduces published changes into current snapshots
- supports direct lookup by entry key
- supports bulk materialization by address and by context
- prunes expired entries when asked

The crate intentionally does not yet own:

- watcher registration or fanout
- visibility evaluation against observers
- transport-neutral wire models
- replication or merge semantics across stores
- cross-resource aggregation into a single subject-level view

## Reduction Rules

1. A store entry is keyed by `(subject, context, resource, origin)`.
2. Publishing an update replaces store metadata for that entry with the update's
   visibility, observed time, and expiry.
3. Facet `Set` operations upsert by facet kind.
4. Facet `Clear` operations remove the matching facet kind.
5. Publish order is authoritative within one store instance. The assigned store
   sequence records that order.
6. Expiry removes whole entries. Partial facet expiry is out of scope for this
   first slice.

## Consequences

This gives `ruthere` a real runtime seam without forcing early answers about
subscriptions or distributed ordering. Higher-level crates can now build
delivery and policy on top of a concrete, tested store.
