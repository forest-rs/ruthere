# watcher_presence_flow

`watcher_presence_flow` is a runnable example for the watcher-oriented
`ruthere` flow.

It demonstrates one narrow loop with concrete key types:

- create a small process-local `PresenceServer`
- configure `PresenceBeacon` publishers with different visibility labels
- publish presence updates into the server from those beacons
- track `WatcherCursor` state for multiple viewers
- poll `has_pending_visible` / `poll_visible`
- refresh subject summaries only when a viewer sees something new
- react to expiry without introducing transport or async machinery

Run it with:

```sh
cargo run -p watcher_presence_flow
```

When run in a terminal, the example uses a lightly styled walkthrough with
sections, indentation, and terminal-aware ANSI color. Set `NO_COLOR=1` to
disable color if desired.
