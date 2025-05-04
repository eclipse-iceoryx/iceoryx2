# Discovery Service

## Running The Example

This example demonstrates how to leverage the discovery service to hook into
discovery updates in the iceoryx2 system.

Three terminals will be required to demonstrate this functionality:

### Terminal 1

Run the service discovery service via the CLI:

```sh
iox2 service discovery
```

### Terminal 2

Run the example, which will subscribe to updates from the service discovery
service:

```sh
cargo run --example discovery_service
```

### Terminal 3

Start another service - it's appearance should be detected in Terminal 1
and reacted to in Terminal 2:

```sh
cargo run --example publish_subscribe_subscriber
```

This demonstrates how any application can hook into changes in the service
landscape of the iceoryx2 system.
