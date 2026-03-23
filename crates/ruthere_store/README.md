# ruthere_store

`ruthere_store` is the first runtime crate for `ruthere`.

It provides an in-memory store over `ruthere_core` updates and snapshots. The
crate owns sequencing, snapshot reduction, and expiry pruning for the local
store.

It also provides a subject-level projection layer that can turn multiple
resource snapshots into a calm subject summary while preserving the underlying
resource detail.

The crate does not yet define subscriptions, watcher policy evaluation, or
transport integration.
