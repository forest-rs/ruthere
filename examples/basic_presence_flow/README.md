# basic_presence_flow

`basic_presence_flow` is a runnable onboarding example for `ruthere`.

It demonstrates the intended end-to-end flow with concrete key types:

- create an in-memory store
- publish presence updates for one subject from multiple resources
- inspect a single materialized snapshot
- materialize all snapshots in a context
- expire stale entries

Run it with:

```sh
cargo run -p basic_presence_flow
```
