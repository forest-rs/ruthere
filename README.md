# R U There?

`ruthere` helps Rust applications answer "who or what is here right now?" in a
way that stays useful once presence stops being a single online/offline bit.

It is for systems where one person, bot, device, shipment, or other subject can
show up in several places at once: a browser tab editing a document, a phone
idling in the same workspace, a tracker reporting nearby luggage, or a public
observer joining a live view. Publishers send small typed updates, and `ruthere`
keeps the current resource-level state plus the subject-level summaries and
watcher cursors that applications usually need.

Non-goals:

- transport, auth, or push delivery
- a wire format or protocol stack
- durable storage
- a policy engine for visibility decisions

## Crates

- `ruthere_core`: `no_std + alloc` semantic types for addresses, timestamps,
  expiry, visibility labels, facets, updates, and snapshots.
- `ruthere_store`: `no_std + alloc` in-memory reduction, retained changes,
  expiry execution, visibility-filtered reads, subject summaries, and watcher
  cursors.
- `ruthere_server`: a small process-local server seam over `ruthere_store` for
  ingest, expiry, and watcher polling.
- `ruthere_beacon`: write-side helpers for building `PresenceUpdate` values
  from stable publication metadata.

The runnable examples live as separate workspace crates under `examples/`.

## Concepts

- subject: the entity whose presence is described.
- context: the scope where that presence is meaningful.
- resource: an optional contributor to a subject's presence in one context,
  such as a device, tracker, shipment, or shared system.
- origin: the publisher or observer that asserted the facts.
- facet: a typed presence fact, either built in or application-defined.
- watcher cursor: a local retained-log sequence position for incremental
  polling.

## Example

```rust
use ruthere_beacon::{ExpiryPolicy, PresenceBeacon};
use ruthere_core::{
    Activity, Availability, PresenceAddress, Timestamp, Visibility,
};
use ruthere_store::InMemoryStore;

let beacon = PresenceBeacon::new(
    PresenceAddress::new("alice", "doc-42", Some("browser-tab")),
    "session/browser",
)
.with_visibility(Visibility::Restricted("doc-members"))
.with_expiry_policy(ExpiryPolicy::After(60));

let mut store = InMemoryStore::<&str, &str, &str, &str, &str>::new();

let sequence = store.publish(
    beacon
        .heartbeat_at(Timestamp::new(100))
        .set_availability(Availability::Available)
        .set_activity(Activity::Editing),
);

let summary = store.subject_summary_in_context(&"alice", &"doc-42");

assert_eq!(sequence, 1);
assert!(summary.is_some());
```

## Examples

Run the examples to see the intended call sites:

```sh
cargo run -p basic_presence_flow
cargo run -p watcher_presence_flow
cargo run -p associated_resource_flow
```

## Extension Points

- Use application key types for subjects, contexts, resources, origins, and
  visibility labels.
- Add typed extension facets with `ExtensionFacet` when built-in availability,
  activity, and last-seen facts are not enough.
- Provide caller-owned visibility policies to filtered store reads and watcher
  polling.
- Build transport, durable storage, subscription identity, or protocol mapping
  in outer crates.

## Gotchas

- Visibility in `ruthere_core` is only a label; callers supply policy.
- Subject summaries are derived projections. Raw per-resource, per-origin
  snapshots remain the canonical stored truth.
- Retained-log compaction can create gaps. A stale watcher must rebuild from
  current materialized state before continuing incremental polling.
- Core crates are intended to stay `no_std` where practical; examples are
  std-only binaries.
