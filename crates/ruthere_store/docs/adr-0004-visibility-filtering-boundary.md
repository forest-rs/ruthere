# ADR-0004: Visibility Filtering Boundary

- Status: Accepted
- Date: 2026-03-23
- Ticket: `rs-4tm4`

## Context

`ruthere_core` carries visibility labels, but the runtime store has so far only
materialized raw snapshots and projected subject summaries without applying any
observer-specific filtering. Callers therefore need to manually inspect labels
after the fact, which defeats the point of having a reusable presence store
seam.

The store needs a way to apply caller-supplied visibility policy while keeping
the policy logic itself outside the crate.

## Decision

`ruthere_store` owns visibility-aware reads and projections, but not visibility
policy.

The crate now provides:

- a `VisibilityPolicy` trait implemented for closures
- filtered snapshot APIs
- filtered subject-summary APIs

The store still does not own:

- watcher identity
- auth/authz policy
- visibility-aware change-log filtering

The change log is intentionally excluded from this slice because expiry changes
currently do not retain enough visibility metadata to filter removals safely.

## Consequences

Callers can now ask the store for "what this viewer may see" without moving
policy into `ruthere_core` or duplicating filtering logic in UI code. The seam
remains caller-controlled and additive to the existing raw APIs.
