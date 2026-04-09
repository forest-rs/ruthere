# ruthere_server

`ruthere_server` is a small process-local server seam for `ruthere`.

It receives `PresenceUpdate` values, applies expiry against an internal
`ruthere_store::InMemoryStore`, and offers watcher polling against that store
without introducing transport, auth, or background fanout.

The crate is intentionally small. It owns ingest and lifecycle helpers, while
still exposing the underlying store read-only for richer snapshot and
projection queries.

It also exposes two narrow contracts for future runtimes:

- `PresenceIngress` for receiving updates and applying expiry
- `PresenceWatch` for watcher cursor creation and gap-aware retained-change polling

The richer store-backed read model intentionally remains on the concrete
`PresenceServer`.

## Example

```rust
use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
use ruthere_core::{PresenceAddress, Timestamp, Visibility};
use ruthere_server::PresenceServer;

let beacon = PresenceBeacon::new(
    PresenceAddress::new("alice", "doc-42", Some("browser-tab")),
    "session/browser",
)
.with_visibility(Visibility::Restricted("doc-members"))
.with_expiry_policy(ExpiryPolicy::After(60));

let mut server = PresenceServer::<&str, &str, &str, &str, &str>::new();
let sequence = server.receive(beacon.heartbeat_at(Timestamp::new(100)));

assert_eq!(sequence, 1);
assert_eq!(server.last_sequence(), 1);
```
