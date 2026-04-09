# associated_resource_flow

`associated_resource_flow` is a runnable example that demonstrates how one
subject can have a calm presence view built from both direct and associated
resources.

It demonstrates:

- one subject in one context with multiple contributing resources
- a direct device resource, a tracker resource, and a shipment resource
- distinct origins for local and federated publishers
- typed example-only extension facets for subject-resource relationship and
  associated status
- a calm subject summary that still preserves raw resource drill-down
- visibility-filtered reads over the same underlying store state

Run it with:

```sh
cargo run -p associated_resource_flow
```
