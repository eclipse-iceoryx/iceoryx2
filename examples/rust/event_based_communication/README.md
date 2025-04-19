# Event-Based Communication

This example demonstrates iceoryx2's event multiplexing mechanism in a more
complex setup. The iceoryx2 `Publisher` and `Subscriber` are integrated into
custom `ExamplePublisher` and `ExampleSubscriber` classes, which also
incorporate an additional iceoryx2 `Notifier` and `Listener`. This setup
enables automatic event emission whenever an `ExamplePublisher` or
`ExampleSubscriber` is created or dropped. Additionally, events are emitted
whenever a new `Sample` is sent or received.

When a `struct` implements `SynchronousMultiplexing`, it can be attached to a
`WaitSet`. Both `ExamplePublisher` and `ExampleSubscriber` implement this
interface by forwarding calls to their underlying `Listener`, which already
provides an implementation of `SynchronousMultiplexing`.

The `WaitSet` notifies the user of the origin of an event notification. The
user can then acquire the `EventId` from the `Listener`. Based on the value of
the `EventId`, the user can identify the specific event that occurred and take
appropriate action.

## How to Run

To see this in action, open two separate terminals and run the following
commands:

### Terminal 1

```sh
cargo run --example event_based_comm_publisher
```

### Terminal 2

```sh
cargo run --example event_based_comm_subscriber
```

Feel free to run multiple publishers or subscribers in parallel.
