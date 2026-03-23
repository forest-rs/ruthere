# ruthere_core

`ruthere_core` is the semantic core for `ruthere`.

It defines `no_std + alloc` types for scoped, time-sensitive presence facts:
keys, timestamps, expiry, visibility labels, built-in facets, extension facets,
updates, and snapshots.

It does not define store sequencing, subscriptions, watcher identity, or policy
evaluation. Those runtime concerns are intended for later crates.
