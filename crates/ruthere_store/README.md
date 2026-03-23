# ruthere_store

`ruthere_store` is the first runtime crate for `ruthere`.

It provides an in-memory store over `ruthere_core` updates and snapshots. The
crate owns sequencing, snapshot reduction, and expiry pruning for the local
store. It does not yet define subscriptions, watcher policy evaluation, or
transport integration.
