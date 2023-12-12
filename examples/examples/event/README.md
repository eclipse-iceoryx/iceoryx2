# Event

## Running The Example

This example offers a practical demonstration of inter-process event signaling
in Iceoryx2. It showcases how one process can signal an event to another
process, allowing for efficient communication.

In this scenario, the 'listener' process waits for incoming events. When an
event arrives, it promptly awakens and reports the [`EventId`] of the received
event. The 'notifier' process, on the other hand, periodically sends
notifications with an incrementing [`EventId`] every second.

To see this in action, open two separate terminals and run the following
commands:

**Terminal 1**

```sh
cargo run --example event_listener
```

**Terminal 2**

```sh
cargo run --example event_notifier
```

Feel free to run multiple listeners or notifiers concurrently to observe how
Iceoryx2 efficiently handles event signaling across processes.
