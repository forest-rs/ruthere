# basic_presence_flow

`basic_presence_flow` is a runnable onboarding example for `ruthere`.

It demonstrates the intended end-to-end flow with concrete key types:

- create an in-memory store
- configure `PresenceBeacon` publishers for one subject across multiple resources
- publish presence updates from those beacons
- inspect retained changes with store-assigned cursors
- inspect a single materialized snapshot
- materialize all snapshots in a context
- project subject-level summaries over resource snapshots
- compact retained changes and observe explicit gap metadata on stale queries
- expire stale entries

Run it with:

```sh
cargo run -p basic_presence_flow
```

When run in a terminal, the example uses a lightly styled walkthrough with
sections, indentation, and terminal-aware ANSI color. Set `NO_COLOR=1` to
disable color if desired.
