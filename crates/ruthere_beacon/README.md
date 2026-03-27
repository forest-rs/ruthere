# ruthere_beacon

`ruthere_beacon` is the write-side helper crate for `ruthere`.

It owns publication ergonomics for one addressed presence source. A beacon
captures stable metadata such as address, origin, visibility, and expiry
policy, then builds ready-to-publish `PresenceUpdate` values on demand.

The crate does not own storage, transport, clocks, background scheduling, or
server behavior.

## Example

```rust
use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
use ruthere_core::{
    Activity, Availability, PresenceAddress, Timestamp, Visibility,
};

let beacon = PresenceBeacon::new(
    PresenceAddress::new("alice", "doc-42", Some("browser-tab")),
    "session/browser",
)
.with_visibility(Visibility::Restricted("doc-members"))
.with_expiry_policy(ExpiryPolicy::After(60));

let update = beacon
    .heartbeat_at(Timestamp::new(100))
    .set_availability(Availability::Available)
    .set_activity(Activity::Editing);

assert_eq!(update.observed_at, Timestamp::new(100));
```
