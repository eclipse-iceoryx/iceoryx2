# Discovery

This example demonstrates how to list all active services within your iceoryx2
application. To get the most out of this example, you can combine it with other
examples, such as the [event example](../event/) or the
[publish-subscribe example](../publish_subscribe/), to see active services in
action.

## How to Run

To begin, let's start some interesting services. Open two terminals and execute
the following commands:

### Terminal 1

```sh
cargo run --example event_listener
```

### Terminal 2

```sh
cargo run --example publish_subscribe_subscriber
```

Once these services are running, you can call the following command:

```sh
cargo run --example discovery
```

This will display the static service details of both the event and the
publish-subscribe service, giving you a comprehensive view of the active
services in your iceoryx2 application.
