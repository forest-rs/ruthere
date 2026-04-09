# ADR-0007: Resource-First Presence and Runtime Evolution

- Status: Accepted
- Date: 2026-04-09
- Ticket: `n/a`

## Context

`ruthere_core` already models presence as scoped facts on `(subject, context,
resource?)` plus a distinct `origin`. `ruthere_store` already stores current
state per `(address, origin)`, retains a local change log, and projects
subject-level summaries from resource snapshots.

Recent design review compared that shape with the IMPP, CPIM, PIDF, and SIMPLE
lineage. The useful lessons are architectural, not protocol-specific. `ruthere`
should learn from the resource-first model, soft-state publication, and
projection discipline without importing SIP, XML, or telecom stack concerns
into the core crates.

The repo needs one durable statement of what the current model means, what it
explicitly does not mean, and which seams future work should extend.

## Decision

`ruthere` adopts a resource-first presence model informed by the presence
standards lineage without adopting that protocol stack.

The workspace boundary is:

- `ruthere_core` owns semantic presence facts, addressed scope, typed facets,
  visibility labels, and expiry labels.
- `ruthere_store` owns local sequencing, reduction, expiry execution, retained
  changes, subject-level projection, and local watcher cursors.
- Raw per-resource, per-origin snapshots remain the canonical stored truth.
- Subject summaries are derived views for callers that want a calm headline,
  not replacements for stored resource detail.
- Future wire formats, subscription lifecycles, watcher identity, and transport
  integration belong in separate crates rather than in `ruthere_core` or the
  in-memory store.

## Goals

- Preserve calm internal semantics while leaving runtime mechanics replaceable.
- Keep one subject's multiple resources visible in the data model.
- Preserve visibility and expiry as first-class concerns.
- Leave room for future runtime layers without forcing speculative APIs into the
  current store surface.

## Non-Goals

- Implementing SIP or SIMPLE.
- Making XML or another wire format the internal source of truth.
- Collapsing resource-level state into a single subject-level status bit.
- Moving transport or watcher identity concerns into `ruthere_core`.

## Key Concepts

- `subject`: the presentity-like entity whose presence is being described.
- `context`: the scope in which that presence is meaningful.
- `resource`: an optional contributor to the subject's presence in one context.
  It may be a direct device, an associated object, a shared thing, or another
  relevant resource.
- `origin`: the publisher or observer that asserted the facts; this is distinct
  from
  `resource`.
- `summary`: a projected subject-level view derived from underlying resource
  snapshots.

## Example

One user in one workspace may have both a browser session and a mobile session.
Those become two distinct addresses with the same `subject` and `context`, but
different `resource` values. In another context, the same subject may also have
an associated tracker or a shipment whose status is relevant to that subject.
If two publishers also report on the browser session, the store still keeps
those entries distinct because `origin` is a separate axis.

Callers may then ask `ruthere_store` for a `SubjectPresenceSummary`. That
summary can headline the browser as dominant because it is actively editing,
while still preserving the mobile snapshot for drill-down. The summary is
therefore a projection over the underlying tuple-like state, not the canonical
state itself.

## Invariants

1. Resource and origin remain distinct dimensions.
2. Per-resource, per-origin snapshots stay directly inspectable after
   projection.
3. Store-global sequence numbers are not treated as per-entry or per-summary
   revision numbers.
4. Resource does not imply ownership; ownership or association semantics belong
   in typed outer vocabularies when needed.
5. Visibility remains part of the presence model even when policy evaluation is
   supplied by outer layers.
6. Extension facets remain typed Rust values rather than stringly typed bags.

## Consequences

Near-term runtime evolution should concentrate on the seams that are already
structurally required:

- preserve per-resource, per-origin truth as the canonical stored state
- keep subject summaries derived and inspectable rather than magical
- make retained-change gaps explicit when compaction prevents exact replay
- broaden examples and docs so `resource` is not implicitly limited to
  subject-owned devices
- defer revisioned views, richer provenance, and freshness helpers until
  concrete consumers justify the added surface area

If interchange is needed later, it should be isolated in a dedicated outer
crate such as `ruthere_wire`, `ruthere_pidf`, or `ruthere_watch`. Those crates
may map the internal semantic model into external representations, but they do
not get to define the internal truth.
