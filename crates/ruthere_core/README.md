# ruthere_core

`ruthere_core` is the semantic core for `ruthere`.

It defines `no_std + alloc` types for scoped, time-sensitive presence facts:
keys, timestamps, expiry, visibility labels, built-in facets, extension facets,
updates, and snapshots.

`PresenceAddress` scopes truth by `subject`, `context`, and optional
`resource`, while `origin` remains a distinct publisher axis on updates and
snapshots. That separation is deliberate: one subject may have multiple
resources and multiple concurrent publishers without collapsing into one
headline status.

`resource` is intentionally broader than "one of my devices." A resource may be
a direct endpoint, an associated object, a shared tracker, a shipment, or
another contributor to the subject's presence view in a given context. `origin`
is provenance for the asserted facts, not an ownership claim.

Any wire format, transport envelope, or subscription runtime belongs in outer
crates rather than in `ruthere_core`.

It does not define store sequencing, subscriptions, watcher identity, or policy
evaluation. Those runtime concerns are intended for later crates.
